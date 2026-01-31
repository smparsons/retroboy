export type GameBoyButton =
    | "Up"
    | "Down"
    | "Left"
    | "Right"
    | "A"
    | "B"
    | "Start"
    | "Select";

export const VALID_BUTTONS: GameBoyButton[] = [
    "Up",
    "Down",
    "Left",
    "Right",
    "A",
    "B",
    "Start",
    "Select",
];

export interface GetScreenshotCommand {
    type: "get_screenshot";
    id: string;
}

export interface PressButtonsCommand {
    type: "press_buttons";
    id: string;
    buttons: GameBoyButton[];
    frames: number;
}

export interface GetStatusCommand {
    type: "get_status";
    id: string;
}

export type BrowserCommand =
    | GetScreenshotCommand
    | PressButtonsCommand
    | GetStatusCommand;

export interface ScreenshotResponse {
    type: "screenshot";
    id: string;
    data: string;
}

export interface ButtonsCompleteResponse {
    type: "buttons_complete";
    id: string;
    frames_executed: number;
}

export interface StatusResponse {
    type: "status";
    id: string;
    playing: boolean;
    gameKey: string | null;
}

export interface ErrorResponse {
    type: "error";
    id: string;
    message: string;
}

export type BrowserResponse =
    | ScreenshotResponse
    | ButtonsCompleteResponse
    | StatusResponse
    | ErrorResponse;
