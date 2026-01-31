#!/usr/bin/env node

import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import { z } from "zod";
import { WebSocketBridge } from "./websocket-bridge.js";
import { VALID_BUTTONS, GameBoyButton } from "./types.js";

const bridge = new WebSocketBridge();
bridge.start();

const server = new McpServer({
    name: "retroboy",
    version: "1.0.0",
});

let requestCounter = 0;
function generateId(): string {
    return `req_${++requestCounter}_${Date.now()}`;
}

server.tool(
    "get_screenshot",
    "Captures the current Game Boy screen as a PNG image. Returns a base64-encoded PNG that can be displayed to see the current game state.",
    {},
    async () => {
        if (!bridge.isConnected()) {
            return {
                content: [
                    {
                        type: "text",
                        text: "Error: Browser not connected. Make sure the RetroBoy web app is running and a game is loaded.",
                    },
                ],
            };
        }

        const response = await bridge.sendCommand({
            type: "get_screenshot",
            id: generateId(),
        });

        if (response.type === "error") {
            return {
                content: [{ type: "text", text: `Error: ${response.message}` }],
            };
        }

        if (response.type === "screenshot") {
            const base64Data = response.data.replace(
                /^data:image\/png;base64,/,
                ""
            );
            return {
                content: [
                    {
                        type: "image",
                        data: base64Data,
                        mimeType: "image/png",
                    },
                ],
            };
        }

        return {
            content: [{ type: "text", text: "Unexpected response type" }],
        };
    }
);

server.tool(
    "press_buttons",
    "Presses one or more Game Boy buttons for a specified number of frames, then releases them. Valid buttons: Up, Down, Left, Right, A, B, Start, Select. Default duration is 10 frames (~167ms at 60fps).",
    {
        buttons: z
            .array(z.enum(VALID_BUTTONS as [GameBoyButton, ...GameBoyButton[]]))
            .describe("Array of buttons to press simultaneously"),
        frames: z
            .number()
            .int()
            .min(1)
            .max(600)
            .default(10)
            .describe(
                "Number of frames to hold the buttons (1-600, default 10)"
            ),
    },
    async ({ buttons, frames }) => {
        if (!bridge.isConnected()) {
            return {
                content: [
                    {
                        type: "text",
                        text: "Error: Browser not connected. Make sure the RetroBoy web app is running and a game is loaded.",
                    },
                ],
            };
        }

        const response = await bridge.sendCommand({
            type: "press_buttons",
            id: generateId(),
            buttons: buttons as GameBoyButton[],
            frames: frames ?? 10,
        });

        if (response.type === "error") {
            return {
                content: [{ type: "text", text: `Error: ${response.message}` }],
            };
        }

        if (response.type === "buttons_complete") {
            return {
                content: [
                    {
                        type: "text",
                        text: `Pressed [${buttons.join(", ")}] for ${response.frames_executed} frames`,
                    },
                ],
            };
        }

        return {
            content: [{ type: "text", text: "Unexpected response type" }],
        };
    }
);

server.tool(
    "get_emulator_status",
    "Returns the current status of the emulator including connection state, whether a game is playing, and the name of the loaded game.",
    {},
    async () => {
        if (!bridge.isConnected()) {
            return {
                content: [
                    {
                        type: "text",
                        text: JSON.stringify(
                            {
                                connected: false,
                                playing: false,
                                gameKey: null,
                            },
                            null,
                            2
                        ),
                    },
                ],
            };
        }

        const response = await bridge.sendCommand({
            type: "get_status",
            id: generateId(),
        });

        if (response.type === "error") {
            return {
                content: [{ type: "text", text: `Error: ${response.message}` }],
            };
        }

        if (response.type === "status") {
            return {
                content: [
                    {
                        type: "text",
                        text: JSON.stringify(
                            {
                                connected: true,
                                playing: response.playing,
                                gameKey: response.gameKey,
                            },
                            null,
                            2
                        ),
                    },
                ],
            };
        }

        return {
            content: [{ type: "text", text: "Unexpected response type" }],
        };
    }
);

async function main() {
    const transport = new StdioServerTransport();
    await server.connect(transport);
}

main().catch(console.error);
