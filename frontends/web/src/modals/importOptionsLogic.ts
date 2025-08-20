export const isValidBase64 = (str: unknown): str is string => {
    if (typeof str !== "string") return false;
    try {
        return btoa(atob(str)) === str;
    } catch {
        return false;
    }
};

export const isValidJson = (value: unknown): boolean => {
    if (typeof value !== "string") return false;
    try {
        const parsed = JSON.parse(value);
        return (
            typeof parsed === "object" &&
            parsed !== null &&
            !Array.isArray(parsed)
        );
    } catch {
        return false;
    }
};

export const isUpperCase = (str: string): boolean => {
    return str === str.toUpperCase() && str !== str.toLowerCase();
};

const buildGeneralSettingsLabel = (
    key: string,
    value: unknown,
): string | undefined => {
    return key === "settings" && isValidJson(value)
        ? "General Settings (Controls/Cheats)"
        : undefined;
};

const buildGameSettingsLabel = (
    key: string,
    value: unknown,
    backupJson: Record<string, unknown>,
): string | undefined => {
    const rtcKey = `${key}-rtc`;
    return isUpperCase(key) && isValidBase64(value)
        ? rtcKey in backupJson && isValidJson(backupJson[rtcKey])
            ? `${key} Cartridge RAM/RTC settings`
            : `${key} Cartridge RAM`
        : undefined;
};

const sortChecklistEntries = (
    entries: [string, string][],
): [string, string][] => {
    const generalSettings = entries.find(([key]) => key === "settings");
    const otherEntries = entries
        .filter(([key]) => key !== "settings")
        .sort(([a], [b]) => a.localeCompare(b));
    return generalSettings ? [generalSettings, ...otherEntries] : otherEntries;
};

export const buildImportOptionsFromBackupJson = (
    backupJson: Record<string, unknown>,
): [string, string][] => {
    const result = Object.entries(backupJson)
        .filter(([key, _]) => !key.endsWith("rtc"))
        .reduce(
            (acc, [key, value]) => {
                const settingsLabel = buildGeneralSettingsLabel(key, value);
                const gameLabel = buildGameSettingsLabel(
                    key,
                    value,
                    backupJson,
                );

                return {
                    ...acc,
                    ...(settingsLabel ? { [key]: settingsLabel } : {}),
                    ...(gameLabel ? { [key]: gameLabel } : {}),
                };
            },
            {} as Record<string, string>,
        );

    return sortChecklistEntries(Object.entries(result));
};
