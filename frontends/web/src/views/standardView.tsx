import FileUploadIcon from "@mui/icons-material/FileUpload";
import FullscreenIcon from "@mui/icons-material/Fullscreen";
import GamepadIcon from "@mui/icons-material/Gamepad";
import PauseIcon from "@mui/icons-material/Pause";
import PhotoCameraIcon from "@mui/icons-material/PhotoCamera";
import PlayArrowIcon from "@mui/icons-material/PlayArrow";
import RefreshIcon from "@mui/icons-material/Refresh";
import VideogameAssetIcon from "@mui/icons-material/VideogameAsset";
import { Button, Typography, styled, Divider } from "@mui/material";
import { RefObject } from "react";

import {
    BufferFileUpload,
    FileBufferObject,
} from "../components/bufferFileUpload";
import { CssGrid, GapSize, Orientation, Position } from "../components/cssGrid";
import GamePad from "../components/gamePad";
import GameScreen from "../components/gameScreen";
import { GameBoyMode, ModeSwitch } from "../components/modeSwitch";
import {
    ResponsiveBreakpoint,
    useResponsiveBreakpoint,
} from "../hooks/useResponsiveBreakpoint";

const AppGrid = styled(CssGrid)`
    height: 100%;
    width: 100%;
`;

const HeaderGrid = styled(CssGrid)`
    margin-bottom: 8px;
`;

const GameSelectionGrid = styled(CssGrid)`
    max-width: 530px;
    margin: 16px;
`;

const GameScreenGrid = styled(CssGrid, {
    shouldForwardProp: prop => prop !== "isMobile",
})<{ isMobile?: boolean }>(({ isMobile }) => ({
    marginBottom: "32px",
    justifySelf: isMobile ? "stretch" : undefined,
    margin: "16px",
}));

const Logo = (): JSX.Element => (
    <img src="/retroboy/logo.png" width="150" height="150" />
);

const StandardView = ({
    gameKey,
    onOpenControls,
    onOpenCheats,
    playing,
    paused,
    mode,
    romBuffer,
    onPlay,
    onPause,
    onResume,
    onScreenshot,
    onReset,
    onFullscreen,
    onModeChange,
    onRomBufferChange,
    canvasRef,
}: StandardViewProps): JSX.Element => {
    const breakpoint = useResponsiveBreakpoint();

    const isMobile = breakpoint === ResponsiveBreakpoint.xs;
    const isTablet = breakpoint === ResponsiveBreakpoint.sm;

    return (
        <AppGrid
            justifyContent={isTablet || isMobile ? undefined : Position.center}
            alignItems={isTablet || isMobile ? Position.end : Position.center}
        >
            <CssGrid
                gap={isTablet || isMobile ? GapSize.large : GapSize.giant}
                alignItems={Position.center}
                justifyItems={Position.center}
                orientation={
                    isTablet || isMobile
                        ? Orientation.vertical
                        : Orientation.horizontal
                }
            >
                <GameSelectionGrid
                    alignItems={Position.end}
                    gap={GapSize.extraLarge}
                >
                    <div>
                        <HeaderGrid
                            orientation={
                                isMobile
                                    ? Orientation.vertical
                                    : Orientation.horizontal
                            }
                            gap={GapSize.large}
                            template={isMobile ? undefined : "1fr auto auto"}
                            justifyContent={
                                isMobile ? Position.stretch : undefined
                            }
                            alignItems={Position.center}
                        >
                            <Logo />
                            <Button
                                variant="contained"
                                color="secondary"
                                startIcon={<GamepadIcon />}
                                onClick={onOpenControls}
                            >
                                Controls
                            </Button>
                            <Button
                                variant="contained"
                                color="secondary"
                                startIcon={<VideogameAssetIcon />}
                                onClick={onOpenCheats}
                                disabled={!gameKey}
                            >
                                Cheats
                            </Button>
                        </HeaderGrid>
                        <Divider />
                    </div>
                    <Typography>
                        Retro Boy is a Game Boy emulator that can be played on
                        the web. To use, simply click "Load ROM" to load your
                        game ROM. Only .gb and .gbc files are supported. Then
                        click "Play".
                    </Typography>
                    <CssGrid
                        orientation={
                            isMobile
                                ? Orientation.vertical
                                : Orientation.horizontal
                        }
                        gap={isMobile ? GapSize.large : undefined}
                        template="1fr auto"
                    >
                        <BufferFileUpload
                            label="Load ROM"
                            onFileSelect={onRomBufferChange}
                            uploadedFile={romBuffer}
                            variant="contained"
                            accept=".gb,.gbc"
                            startIcon={<FileUploadIcon />}
                        />
                        <ModeSwitch
                            disabled={playing || paused}
                            mode={mode}
                            onModeChange={onModeChange}
                        />
                    </CssGrid>
                    <CssGrid
                        orientation={
                            isMobile
                                ? Orientation.vertical
                                : Orientation.horizontal
                        }
                        gap={isMobile ? GapSize.large : GapSize.medium}
                        justifyContent={
                            isMobile ? Position.stretch : Position.start
                        }
                    >
                        {!playing || paused ? (
                            <Button
                                variant="contained"
                                disabled={!romBuffer}
                                onClick={paused ? onResume : onPlay}
                                startIcon={<PlayArrowIcon />}
                            >
                                {paused ? "Resume" : "Play"}
                            </Button>
                        ) : (
                            <Button
                                variant="contained"
                                onClick={onPause}
                                startIcon={<PauseIcon />}
                            >
                                Pause
                            </Button>
                        )}
                        <Button
                            variant="contained"
                            onClick={onReset}
                            disabled={!playing && !paused}
                            startIcon={<RefreshIcon />}
                        >
                            Reset
                        </Button>
                        <Button
                            variant="contained"
                            onClick={onFullscreen}
                            disabled={!playing}
                            startIcon={<FullscreenIcon />}
                        >
                            Fullscreen
                        </Button>
                    </CssGrid>
                </GameSelectionGrid>
                <GameScreenGrid
                    gap={GapSize.large}
                    orientation={Orientation.vertical}
                    justifyItems={isMobile ? undefined : Position.start}
                    isMobile={isMobile}
                >
                    <Button
                        startIcon={<PhotoCameraIcon />}
                        onClick={onScreenshot}
                        disabled={!playing && !paused}
                        color="secondary"
                        variant="contained"
                    >
                        Screenshot
                    </Button>
                    <GameScreen
                        playing={playing}
                        paused={paused}
                        ref={canvasRef}
                        fullscreen={false}
                    />
                </GameScreenGrid>
            </CssGrid>
            {(isTablet || isMobile) && <GamePad playing={playing} />}
        </AppGrid>
    );
};

interface StandardViewProps {
    readonly gameKey: string | null;
    readonly playing: boolean;
    readonly paused: boolean;
    readonly mode: GameBoyMode;
    readonly romBuffer: FileBufferObject | null;
    readonly onOpenControls: () => void;
    readonly onOpenCheats: () => void;
    readonly onPlay: () => void;
    readonly onPause: () => void;
    readonly onResume: () => void;
    readonly onScreenshot: () => void;
    readonly onReset: () => void;
    readonly onFullscreen: () => void;
    readonly onModeChange: (mode: GameBoyMode) => void;
    readonly onRomBufferChange: (romBuffer: FileBufferObject | null) => void;
    readonly canvasRef: RefObject<HTMLCanvasElement>;
}

export default StandardView;
