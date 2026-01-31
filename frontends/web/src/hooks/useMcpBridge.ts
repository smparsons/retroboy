import { MutableRefObject, useEffect, useRef } from "react";

import { ScheduleFrameCallback } from "./useAudioSync";

import { pressKey, releaseKey } from "../core/retroboyCore";

const MCP_WEBSOCKET_URL = "ws://localhost:8765";

type GameBoyButton =
    | "Up"
    | "Down"
    | "Left"
    | "Right"
    | "A"
    | "B"
    | "Start"
    | "Select";

interface GetScreenshotCommand {
    type: "get_screenshot";
    id: string;
}

interface PressButtonsCommand {
    type: "press_buttons";
    id: string;
    buttons: GameBoyButton[];
    frames: number;
}

interface GetStatusCommand {
    type: "get_status";
    id: string;
}

type BrowserCommand =
    | GetScreenshotCommand
    | PressButtonsCommand
    | GetStatusCommand;

const useMcpBridge = (
    playing: boolean,
    gameKey: string | null,
    canvasRef: MutableRefObject<HTMLCanvasElement | null>,
    scheduleFrameCallback: ScheduleFrameCallback,
): void => {
    const wsRef = useRef<WebSocket | null>(null);
    const reconnectTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(
        null,
    );

    const playingRef = useRef(playing);
    const gameKeyRef = useRef(gameKey);
    const scheduleFrameCallbackRef = useRef(scheduleFrameCallback);

    useEffect(() => {
        playingRef.current = playing;
    }, [playing]);

    useEffect(() => {
        gameKeyRef.current = gameKey;
    }, [gameKey]);

    useEffect(() => {
        scheduleFrameCallbackRef.current = scheduleFrameCallback;
    }, [scheduleFrameCallback]);

    useEffect(() => {
        const sendResponse = (response: Record<string, unknown>): void => {
            if (wsRef.current?.readyState === WebSocket.OPEN) {
                wsRef.current.send(JSON.stringify(response));
            }
        };

        const handleGetScreenshot = (command: GetScreenshotCommand): void => {
            if (!canvasRef.current) {
                sendResponse({
                    type: "error",
                    id: command.id,
                    message: "Canvas not available",
                });
                return;
            }

            const dataUrl = canvasRef.current.toDataURL("image/png");
            sendResponse({
                type: "screenshot",
                id: command.id,
                data: dataUrl,
            });
        };

        const handlePressButtons = (command: PressButtonsCommand): void => {
            if (!playingRef.current) {
                sendResponse({
                    type: "error",
                    id: command.id,
                    message: "Game not playing",
                });
                return;
            }

            for (const button of command.buttons) {
                pressKey(button);
            }

            scheduleFrameCallbackRef.current(command.frames, () => {
                for (const button of command.buttons) {
                    releaseKey(button);
                }
                sendResponse({
                    type: "buttons_complete",
                    id: command.id,
                    frames_executed: command.frames,
                });
            });
        };

        const handleGetStatus = (command: GetStatusCommand): void => {
            sendResponse({
                type: "status",
                id: command.id,
                playing: playingRef.current,
                gameKey: gameKeyRef.current,
            });
        };

        const handleCommand = (command: BrowserCommand): void => {
            switch (command.type) {
                case "get_screenshot":
                    handleGetScreenshot(command);
                    break;
                case "press_buttons":
                    handlePressButtons(command);
                    break;
                case "get_status":
                    handleGetStatus(command);
                    break;
            }
        };

        const connect = (): void => {
            if (wsRef.current?.readyState === WebSocket.OPEN) return;

            try {
                const ws = new WebSocket(MCP_WEBSOCKET_URL);

                ws.onopen = () => {
                    console.log("MCP bridge connected");
                };

                ws.onmessage = event => {
                    try {
                        const command = JSON.parse(
                            event.data as string,
                        ) as BrowserCommand;
                        handleCommand(command);
                    } catch {
                        console.error("Failed to parse MCP command");
                    }
                };

                ws.onclose = () => {
                    wsRef.current = null;
                    reconnectTimeoutRef.current = setTimeout(connect, 3000);
                };

                ws.onerror = () => {
                    ws.close();
                };

                wsRef.current = ws;
            } catch {
                reconnectTimeoutRef.current = setTimeout(connect, 3000);
            }
        };

        connect();

        return () => {
            if (reconnectTimeoutRef.current) {
                clearTimeout(reconnectTimeoutRef.current);
            }
            wsRef.current?.close();
        };
    }, []);
};

export default useMcpBridge;
