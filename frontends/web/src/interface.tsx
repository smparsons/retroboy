import { useEffect, useRef, useState } from "react";

import { FileBufferObject } from "./components/bufferFileUpload";
import { gameBoyModes } from "./components/modeSwitch";
import {
    decodeEmulatorState,
    EmulatorSettings,
    encodeEmulatorState,
    initializeEmulator,
    registerGamegenieCheat,
    registerGamesharkCheat,
    resetEmulator,
} from "./core/retroboyCore";
import useAudioSync from "./hooks/useAudioSync";
import { useKeyListeners } from "./hooks/useKeyListeners";
import { useIsMobile } from "./hooks/useResponsiveBreakpoint";
import { useSettingsStore } from "./hooks/useSettingsStore";
import { useTopLevelRenderer } from "./hooks/useTopLevelRenderer";
import { CheatType } from "./modals/cheatsModal";
import { CheatsModal } from "./modals/cheatsModal";
import { ControlsModal } from "./modals/controlsModal";
import { MessageDialog } from "./modals/dialog";
import FullscreenView from "./views/fullscreenView";
import StandardView from "./views/standardView";

const errorModalKey = "error-modal";
const controlsModalKey = "controls-modal";
const cheatsModalKey = "cheats-modal";

const Interface = (): JSX.Element => {
    const isMobile = useIsMobile();

    const { displayTopLevelComponent, removeTopLevelComponent } =
        useTopLevelRenderer();

    const [romBuffer, setRomBuffer] = useState(null as FileBufferObject | null);

    const [gameKey, setGameKey] = useState(null as string | null);
    const [playing, setPlaying] = useState(false);
    const [paused, setPaused] = useState(false);
    const [mode, setMode] = useState(gameBoyModes.dmg);
    const [fullscreenMode, setFullscreenMode] = useState(false);
    const [usingModal, setUsingModal] = useState(false);

    useKeyListeners(playing, usingModal);

    const canvasRef = useRef<HTMLCanvasElement | null>(null);

    const resetGame = (): void => {
        setGameKey(null);
        setPlaying(false);
        setPaused(false);
        resetEmulator();
        setRomBuffer(null);
    };

    const [audioContextRef, startReset] = useAudioSync(playing, resetGame);

    const scrollToGamePad = ({ smooth }: { smooth: boolean }) => {
        window.scrollTo({
            top: document.body.scrollHeight,
            behavior: smooth ? "smooth" : "auto",
        });
    };

    const { settings } = useSettingsStore();

    const registerCheats = (newGameKey: string): void => {
        const cheatsById = settings.cheats ? settings.cheats[newGameKey] : {};
        const cheats = Object.values(cheatsById || {});
        cheats.forEach(cheat => {
            if (cheat.registered) {
                if (cheat.type == CheatType.GameShark) {
                    registerGamesharkCheat(cheat.id, cheat.code);
                } else {
                    registerGamegenieCheat(cheat.id, cheat.code);
                }
            }
        });
    };

    const playGame = () => {
        if (romBuffer) {
            if (!audioContextRef.current) {
                audioContextRef.current = new AudioContext();
            }

            const emulatorSettings = new EmulatorSettings(
                mode,
                audioContextRef.current.sampleRate,
            );
            const { error, metadata } = initializeEmulator(
                romBuffer.data,
                emulatorSettings,
            );

            if (error) {
                openErrorDialog(error);
                resetGame();
            } else if (metadata) {
                setGameKey(metadata.title);
                registerCheats(metadata.title);
                setPlaying(true);
                scrollToGamePad({ smooth: true });
            }
        }
    };

    const pauseGame = (): void => {
        setPaused(true);
        setPlaying(false);
    };

    const resumeGame = (): void => {
        setPaused(false);
        setPlaying(true);
        scrollToGamePad({ smooth: true });
    };

    const setFullscreen = (): void => {
        setFullscreenMode(true);
        if (document.body && document.body.requestFullscreen) {
            document.body.requestFullscreen();
        }
    };

    const onFullscreenChange = (): void => {
        if (!document.fullscreenElement) {
            exitFullscreen();
        }
    };

    const exitFullscreen = (): void => {
        setFullscreenMode(false);
        if (document.exitFullscreen) {
            document.exitFullscreen().catch(() => {});
        }
    };

    const downloadScreenshot = (): void => {
        if (canvasRef.current) {
            const dataUrl = canvasRef.current.toDataURL("image/png");
            const link = document.createElement("a");
            link.href = dataUrl;
            link.download = "retroboy-screenshot.png";
            link.click();
        }
    };

    const openErrorDialog = (message: string): void => {
        displayTopLevelComponent(
            errorModalKey,
            <MessageDialog
                heading="Error"
                message={message}
                onClose={() => removeTopLevelComponent(errorModalKey)}
            />,
        );
    };

    const openControls = (): void => {
        setUsingModal(true);
        displayTopLevelComponent(
            controlsModalKey,
            <ControlsModal
                onClose={() => {
                    removeTopLevelComponent(controlsModalKey);
                    setUsingModal(false);
                }}
            />,
        );
    };

    const handleRomBufferChange = (
        fileObject: FileBufferObject | null,
    ): void => {
        if (
            fileObject?.filename.endsWith(".gbc") &&
            mode === gameBoyModes.dmg
        ) {
            setMode(gameBoyModes.cgb);
        }
        setRomBuffer(fileObject);
    };

    const openCheats = (): void => {
        if (gameKey) {
            setUsingModal(true);
            displayTopLevelComponent(
                cheatsModalKey,
                <CheatsModal
                    gameKey={gameKey}
                    onClose={() => {
                        removeTopLevelComponent(cheatsModalKey);
                        setUsingModal(false);
                    }}
                />,
            );
        }
    };

    const importSaveState = (): void => {
        if (gameKey) {
            const input = document.createElement("input");
            input.type = "file";
            input.accept = ".rbs";
            input.onchange = async () => {
                const file = input.files?.[0];
                if (file) {
                    const reader = new FileReader();
                    reader.onload = async () => {
                        const error = decodeEmulatorState(
                            new Uint8Array(reader.result as ArrayBuffer),
                        );
                        if (error) {
                            openErrorDialog(error);
                        }
                    };
                    reader.readAsArrayBuffer(file);
                }
            };
            input.click();
        }
    };

    const exportSaveState = (): void => {
        if (gameKey) {
            const result = encodeEmulatorState();
            if (result.error) {
                openErrorDialog(result.error);
            } else if (result.saveState) {
                const blob = new Blob([result.saveState], {
                    type: "application/octet-stream",
                });
                const url = URL.createObjectURL(blob);
                const a = document.createElement("a");
                a.href = url;
                a.download = "save_state.rbs";
                a.click();
                URL.revokeObjectURL(url);
            }
        }
    };

    useEffect(() => {
        if (playing && !fullscreenMode && isMobile) {
            setTimeout(() => {
                window.requestAnimationFrame(() =>
                    scrollToGamePad({ smooth: false }),
                );
            });
        }
    }, [fullscreenMode]);

    useEffect(() => {
        document.addEventListener("fullscreenchange", onFullscreenChange);
        return () => {
            document.removeEventListener(
                "fullscreenchange",
                onFullscreenChange,
            );
        };
    }, []);

    return fullscreenMode ? (
        <FullscreenView
            playing={playing}
            paused={paused}
            onExitFullscreen={exitFullscreen}
            canvasRef={canvasRef}
        />
    ) : (
        <StandardView
            gameKey={gameKey}
            playing={playing}
            paused={paused}
            romBuffer={romBuffer}
            mode={mode}
            onRomBufferChange={handleRomBufferChange}
            onModeChange={setMode}
            onPlay={playGame}
            onPause={pauseGame}
            onResume={resumeGame}
            onReset={startReset}
            onFullscreen={setFullscreen}
            onScreenshot={downloadScreenshot}
            onOpenControls={openControls}
            onOpenCheats={openCheats}
            onImportSaveState={importSaveState}
            onExportSaveState={exportSaveState}
            canvasRef={canvasRef}
        />
    );
};

export default Interface;
