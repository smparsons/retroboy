import { MutableRefObject, useCallback, useEffect, useRef } from "react";

import { stepUntilNextAudioBuffer } from "../core/retroboyCore";

export type FrameCallback = {
    targetFrame: number;
    callback: () => void;
};

export type ScheduleFrameCallback = (
    framesFromNow: number,
    callback: () => void,
) => void;

const useAudioSync = (
    playing: boolean,
    resetGameCallback: () => void,
): [
    MutableRefObject<AudioContext | null>,
    () => void,
    MutableRefObject<number>,
    ScheduleFrameCallback,
] => {
    const audioContextRef = useRef<AudioContext | null>(null);
    const scheduledResetRef = useRef<boolean>(false);
    const frameCountRef = useRef<number>(0);
    const frameCallbacksRef = useRef<FrameCallback[]>([]);

    const nextPlayTimeRef = useRef<number>(0);

    const resetGame = (): void => {
        resetGameCallback();
        scheduledResetRef.current = false;
    };

    const startReset = (): void => {
        if (playing) {
            scheduledResetRef.current = true;
        } else {
            resetGame();
        }
    };

    const scheduleFrameCallback: ScheduleFrameCallback = useCallback(
        (framesFromNow: number, callback: () => void) => {
            const targetFrame = frameCountRef.current + framesFromNow;
            frameCallbacksRef.current.push({ targetFrame, callback });
        },
        [],
    );

    const processFrameCallbacks = useCallback(() => {
        const currentFrame = frameCountRef.current;
        const remaining: FrameCallback[] = [];

        for (const entry of frameCallbacksRef.current) {
            if (currentFrame >= entry.targetFrame) {
                entry.callback();
            } else {
                remaining.push(entry);
            }
        }

        frameCallbacksRef.current = remaining;
    }, []);

    const step = useCallback(() => {
        if (scheduledResetRef.current) {
            resetGame();
        } else if (playing) {
            stepUntilNextAudioBuffer();
            frameCountRef.current++;
            processFrameCallbacks();
        }
    }, [playing, processFrameCallbacks]);

    useEffect(() => {
        if (playing && audioContextRef.current) {
            nextPlayTimeRef.current = audioContextRef.current.currentTime;
        }
    }, [playing]);

    const GAP_BEFORE_SAMPLE_PLAY = 15;

    useEffect(() => {
        (window as any).playAudioSamples = (
            leftAudioSamples: number[],
            rightAudioSamples: number[],
        ): void => {
            const audioContext = audioContextRef.current;

            if (audioContext) {
                const bufferLength = leftAudioSamples.length;
                if (bufferLength === 0) {
                    return;
                }
                const audioBuffer = audioContext.createBuffer(
                    2,
                    bufferLength,
                    audioContext.sampleRate,
                );

                const leftChannel = audioBuffer.getChannelData(0);
                const rightChannel = audioBuffer.getChannelData(1);

                for (let i = 0; i < bufferLength; i++) {
                    leftChannel[i] = leftAudioSamples[i];
                    rightChannel[i] = rightAudioSamples[i];
                }

                const bufferSource = audioContext.createBufferSource();
                bufferSource.buffer = audioBuffer;

                const duration = bufferLength / audioContext.sampleRate;

                bufferSource.connect(audioContext.destination);

                bufferSource.start(nextPlayTimeRef.current);

                const waitTime =
                    (nextPlayTimeRef.current - audioContext.currentTime) * 1000;

                setTimeout(step, waitTime - GAP_BEFORE_SAMPLE_PLAY);

                nextPlayTimeRef.current = nextPlayTimeRef.current
                    ? nextPlayTimeRef.current + duration
                    : duration;
            }
        };
    }, [playing]);

    useEffect(() => {
        if (playing) {
            step();
        }
    }, [playing]);

    return [audioContextRef, startReset, frameCountRef, scheduleFrameCallback];
};

export default useAudioSync;
