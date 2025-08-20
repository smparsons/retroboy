import { buildImportOptionsFromBackupJson } from "./importOptionsLogic";

describe("buildChecklistFromBackupJson", () => {
    it("should find general settings and create display text for it", () => {
        const backupJson = {
            settings: JSON.stringify({ controls: "gamepad", cheats: true }),
        };

        const result = buildImportOptionsFromBackupJson(backupJson);

        expect(result).toEqual([
            ["settings", "General Settings (Controls/Cheats)"],
        ]);
    });

    it("should find a game key with cartridge RAM and create display text for it", () => {
        const validBase64 = btoa("test cartridge data");
        const backupJson = {
            POKEMON: validBase64,
        };

        const result = buildImportOptionsFromBackupJson(backupJson);

        expect(result).toEqual([["POKEMON", "POKEMON Cartridge RAM"]]);
    });

    it("should find a game key with cartridge RAM and RTC settings and create a single key/value pair with display text for it", () => {
        const validBase64 = btoa("test cartridge data");
        const backupJson = {
            POKEMON: validBase64,
            "POKEMON-rtc": JSON.stringify({ rtcData: "some rtc data" }),
        };

        const result = buildImportOptionsFromBackupJson(backupJson);

        expect(result).toEqual([
            ["POKEMON", "POKEMON Cartridge RAM/RTC settings"],
        ]);
    });

    it("should not process a game key if the key is not in all upper case", () => {
        const validBase64 = btoa("test cartridge data");
        const backupJson = {
            pokemon: validBase64,
            MixedCase: validBase64,
            UPPERCASE: validBase64,
        };

        const result = buildImportOptionsFromBackupJson(backupJson);

        expect(result).toEqual([["UPPERCASE", "UPPERCASE Cartridge RAM"]]);
    });

    it("should not process a game key if the value is not a valid base 64 string", () => {
        const backupJson = {
            POKEMON: "invalid base64 string!@#",
            ZELDA: "also invalid",
            MARIO: btoa("valid base64 data"),
        };

        const result = buildImportOptionsFromBackupJson(backupJson);

        expect(result).toEqual([["MARIO", "MARIO Cartridge RAM"]]);
    });

    it("should not process general settings if the value is not a valid JSON object", () => {
        const backupJson = {
            settings: "invalid json string",
            POKEMON: btoa("valid base64 data"),
        };

        const result = buildImportOptionsFromBackupJson(backupJson);

        expect(result).toEqual([["POKEMON", "POKEMON Cartridge RAM"]]);
    });

    it("should not process settings if the value is an array string", () => {
        const backupJson = {
            settings: JSON.stringify(["array", "is", "not", "valid"]),
            POKEMON: btoa("valid base64 data"),
        };

        const result = buildImportOptionsFromBackupJson(backupJson);

        expect(result).toEqual([["POKEMON", "POKEMON Cartridge RAM"]]);
    });

    it("should not process settings if the value is null string", () => {
        const backupJson = {
            settings: JSON.stringify(null),
            POKEMON: btoa("valid base64 data"),
        };

        const result = buildImportOptionsFromBackupJson(backupJson);

        expect(result).toEqual([["POKEMON", "POKEMON Cartridge RAM"]]);
    });

    it("should not duplicate entries when processing both cartridge RAM and RTC", () => {
        const validBase64 = btoa("test cartridge data");
        const backupJson = {
            "POKEMON-rtc": JSON.stringify({ rtcData: "some rtc data" }),
            POKEMON: validBase64,
        };

        const result = buildImportOptionsFromBackupJson(backupJson);

        expect(result).toEqual([
            ["POKEMON", "POKEMON Cartridge RAM/RTC settings"],
        ]);
        expect(result).toHaveLength(1);
    });

    it("should handle complex mixed scenarios", () => {
        const validBase64 = btoa("test cartridge data");
        const backupJson = {
            settings: JSON.stringify({ controls: "gamepad" }),
            POKEMON: validBase64,
            "POKEMON-rtc": JSON.stringify({ rtcData: "data" }),
            ZELDA: validBase64,
            lowercase: validBase64,
            INVALID: "not base64",
            badSettings: "not json",
        };

        const result = buildImportOptionsFromBackupJson(backupJson);

        expect(result).toEqual([
            ["settings", "General Settings (Controls/Cheats)"],
            ["POKEMON", "POKEMON Cartridge RAM/RTC settings"],
            ["ZELDA", "ZELDA Cartridge RAM"],
        ]);
    });
});
