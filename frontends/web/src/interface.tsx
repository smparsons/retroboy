import { useEffect, useRef, useState } from "react";

import {
    buildFileBufferObject,
    FileBufferObject,
} from "./components/bufferFileUpload";
import { gameBoyModes } from "./components/modeSwitch";
import {
    applySaveState,
    EmulatorSettings,
    encodeSaveState,
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
import { ImportBackupModal } from "./modals/importBackupModal";
import { buildImportOptionsFromBackupJson } from "./modals/importOptionsLogic";
import FullscreenView from "./views/fullscreenView";
import StandardView from "./views/standardView";

const errorModalKey = "error-modal";
const controlsModalKey = "controls-modal";
const cheatsModalKey = "cheats-modal";
const importBackupModalKey = "import-backup-modal";

const Interface = (): JSX.Element => {
    const isMobile = useIsMobile();

    const { displayTopLevelComponent, removeTopLevelComponent } =
        useTopLevelRenderer();

    const [rom, setRom] = useState(null as FileBufferObject | null);

    const [gameKey, setGameKey] = useState(null as string | null);
    const [playing, setPlaying] = useState(false);
    const [paused, setPaused] = useState(false);
    const [mode, setMode] = useState(gameBoyModes.dmg);
    const [fullscreenMode, setFullscreenMode] = useState(false);
    const [usingModal, setUsingModal] = useState(false);

    useKeyListeners(playing, usingModal);

    const canvasRef = useRef<HTMLCanvasElement | null>(null);
    const loadStateRef = useRef<HTMLInputElement>(null);
    const importBackupRef = useRef<HTMLInputElement>(null);

    const resetGame = (): void => {
        setGameKey(null);
        setPlaying(false);
        setPaused(false);
        resetEmulator();
        setRom(null);
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
        if (rom) {
            if (!audioContextRef.current) {
                audioContextRef.current = new AudioContext();
            }

            const emulatorSettings = new EmulatorSettings(
                mode,
                audioContextRef.current.sampleRate,
            );
            const { error, metadata } = initializeEmulator(
                rom.data,
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

    const handleRomChange = (fileObject: FileBufferObject | null): void => {
        if (
            fileObject?.filename.endsWith(".gbc") &&
            mode === gameBoyModes.dmg
        ) {
            setMode(gameBoyModes.cgb);
        }
        setRom(fileObject);
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
    const handleFileUpload = async (
        event: React.ChangeEvent<HTMLInputElement>,
        fileUploadRef: React.RefObject<HTMLInputElement>,
        callback: (file: File) => Promise<void>,
    ): Promise<void> => {
        const fileList = event.target.files;
        if (!fileList) return;
        const file = fileList[0];
        if (file) {
            await callback(file);
            if (fileUploadRef.current) {
                fileUploadRef.current.value = "";
            }
        }
    };

    const downloadFile = (
        content: BlobPart,
        filename: string,
        mimeType: string,
    ): void => {
        const blob = new Blob([content], { type: mimeType });
        const url = URL.createObjectURL(blob);
        const link = document.createElement("a");
        link.href = url;
        link.download = filename;
        link.click();
        URL.revokeObjectURL(url);
    };

    const handleLoadStateChange = (
        event: React.ChangeEvent<HTMLInputElement>,
    ): Promise<void> => {
        return handleFileUpload(event, loadStateRef, async (file: File) => {
            try {
                const bufferObject = await buildFileBufferObject(file);
                const error = applySaveState(bufferObject.data);
                if (error) {
                    openErrorDialog(error);
                }
            } catch (err) {
                openErrorDialog(
                    "An error occurred while loading the save state.",
                );
                console.error("Error loading save state:", err);
            }
        });
    };

    const saveState = (): void => {
        if (gameKey && rom) {
            const result = encodeSaveState();
            if (result.error) {
                openErrorDialog(result.error);
            } else if (result.saveState) {
                const saveStateName = rom.filename.split(".")[0];
                const filename = `${saveStateName}.rbs`;
                downloadFile(
                    result.saveState,
                    filename,
                    "application/octet-stream",
                );
            }
        }
    };

    const openImportBackup = (backupJson: Record<string, unknown>): void => {
        setUsingModal(true);
        const importOptions = buildImportOptionsFromBackupJson(backupJson);
        if (!importOptions.length) {
            openErrorDialog("This JSON file has no valid data to import.");
        } else {
            displayTopLevelComponent(
                importBackupModalKey,
                <ImportBackupModal
                    onClose={() => {
                        removeTopLevelComponent(importBackupModalKey);
                        setUsingModal(false);
                    }}
                    importOptions={importOptions}
                    backupJson={backupJson}
                />,
            );
        }
    };

    const handleImportBackupChange = async (
        event: React.ChangeEvent<HTMLInputElement>,
    ): Promise<void> => {
        return handleFileUpload(event, importBackupRef, async (file: File) => {
            try {
                const text = await file.text();
                const jsonData = JSON.parse(text) as Record<string, unknown>;
                openImportBackup(jsonData);
            } catch (err) {
                openErrorDialog(
                    "Invalid JSON file. Please select a valid backup file.",
                );
                console.error("Error parsing JSON:", err);
            }
        });
    };

    const exportBackup = (): void => {
        const backupData: Record<string, unknown> = {};

        if (!localStorage.length) {
            openErrorDialog("You currently have no stored settings to export.");
        } else {
            for (let i = 0; i < localStorage.length; i++) {
                const key = localStorage.key(i);
                if (key) {
                    const value = localStorage.getItem(key);
                    backupData[key] = value;
                }
            }

            const jsonString = JSON.stringify(backupData, null, 2);
            downloadFile(
                jsonString,
                "retroboy-backup.json",
                "application/json",
            );
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
            rom={rom}
            mode={mode}
            onRomChange={handleRomChange}
            onModeChange={setMode}
            onPlay={playGame}
            onPause={pauseGame}
            onResume={resumeGame}
            onReset={startReset}
            onFullscreen={setFullscreen}
            onScreenshot={downloadScreenshot}
            onOpenControls={openControls}
            onOpenCheats={openCheats}
            onLoadStateChange={handleLoadStateChange}
            onSaveState={saveState}
            onImportBackupChange={handleImportBackupChange}
            onExportBackup={exportBackup}
            canvasRef={canvasRef}
            loadStateRef={loadStateRef}
            importBackupRef={importBackupRef}
        />
    );
};

export default Interface;
