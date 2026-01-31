import { WebSocketServer, WebSocket } from "ws";
import { BrowserCommand, BrowserResponse } from "./types.js";

const WEBSOCKET_PORT = 8765;

type PendingRequest = {
    resolve: (response: BrowserResponse) => void;
    reject: (error: Error) => void;
    timeout: ReturnType<typeof setTimeout>;
};

export class WebSocketBridge {
    private wss: WebSocketServer | null = null;
    private browserConnection: WebSocket | null = null;
    private pendingRequests: Map<string, PendingRequest> = new Map();
    private requestTimeout = 30000;

    start(): void {
        if (this.wss) return;

        this.wss = new WebSocketServer({ port: WEBSOCKET_PORT });

        this.wss.on("connection", (ws) => {
            if (this.browserConnection) {
                this.browserConnection.close();
            }
            this.browserConnection = ws;

            ws.on("message", (data) => {
                try {
                    const response = JSON.parse(data.toString()) as BrowserResponse;
                    this.handleResponse(response);
                } catch {
                    console.error("Failed to parse browser message");
                }
            });

            ws.on("close", () => {
                if (this.browserConnection === ws) {
                    this.browserConnection = null;
                    this.rejectAllPending("Browser disconnected");
                }
            });

            ws.on("error", () => {
                if (this.browserConnection === ws) {
                    this.browserConnection = null;
                    this.rejectAllPending("Browser connection error");
                }
            });
        });
    }

    stop(): void {
        this.rejectAllPending("Server shutting down");
        this.browserConnection?.close();
        this.wss?.close();
        this.wss = null;
        this.browserConnection = null;
    }

    isConnected(): boolean {
        return (
            this.browserConnection !== null &&
            this.browserConnection.readyState === WebSocket.OPEN
        );
    }

    async sendCommand(command: BrowserCommand): Promise<BrowserResponse> {
        if (!this.isConnected()) {
            throw new Error("Browser not connected");
        }

        return new Promise((resolve, reject) => {
            const timeout = setTimeout(() => {
                this.pendingRequests.delete(command.id);
                reject(new Error("Request timeout"));
            }, this.requestTimeout);

            this.pendingRequests.set(command.id, { resolve, reject, timeout });
            this.browserConnection!.send(JSON.stringify(command));
        });
    }

    private handleResponse(response: BrowserResponse): void {
        const pending = this.pendingRequests.get(response.id);
        if (pending) {
            clearTimeout(pending.timeout);
            this.pendingRequests.delete(response.id);
            pending.resolve(response);
        }
    }

    private rejectAllPending(reason: string): void {
        for (const [id, pending] of this.pendingRequests) {
            clearTimeout(pending.timeout);
            pending.reject(new Error(reason));
        }
        this.pendingRequests.clear();
    }
}
