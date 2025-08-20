import { Typography } from "@mui/material";

import {
    isValidJson,
    buildImportOptionsFromBackupJson,
} from "./importOptionsLogic";

import FieldCheckbox from "../components/form/fieldCheckbox";
import FormModal from "../components/formModal";
import { ListGrid } from "../components/list";
import { useIsMobile } from "../hooks/useResponsiveBreakpoint";
import { useSettingsStore } from "../hooks/useSettingsStore";
import { useToastRenderer } from "../hooks/useToastRenderer";

const checkboxStyles = {
    padding: "4px 8px",
};

export const ImportBackupModal = ({
    onClose,
    importOptions,
    backupJson,
}: ImportBackupModalProps): JSX.Element => {
    const isMobile = useIsMobile();
    const { storeSettings } = useSettingsStore();
    const toast = useToastRenderer();

    const importSettings = (key: string): void => {
        if (backupJson[key] !== undefined && isValidJson(backupJson[key])) {
            const settingsData = JSON.parse(backupJson[key] as string);
            storeSettings(settingsData);
        }
    };

    const importGameData = (key: string): void => {
        if (backupJson[key] !== undefined) {
            localStorage.setItem(key, backupJson[key] as string);
        }

        const rtcKey = `${key}-rtc`;
        if (backupJson[rtcKey] !== undefined) {
            localStorage.setItem(rtcKey, backupJson[rtcKey] as string);
        }
    };

    const handleSend = (
        closeDialog: () => void,
        values: Record<string, unknown>,
    ): Record<string, string> | undefined => {
        const importOptions = values.importOptions as Record<string, boolean>;

        for (const [key, isSelected] of Object.entries(importOptions)) {
            if (isSelected) {
                if (key === "settings") {
                    importSettings(key);
                } else {
                    importGameData(key);
                }
            }
        }

        toast.success("All selected settings have imported successfully!");

        closeDialog();
        return undefined;
    };

    const initialValues = {
        importOptions: Object.fromEntries(
            importOptions.map(([key]) => [key, true]),
        ),
    };

    return (
        <FormModal
            title="Import Backup"
            onClose={onClose}
            submitButtonText="Import"
            cancelButtonText="Close"
            onCancel={onClose}
            initialValues={initialValues}
            onSubmit={({ values }) => handleSend(onClose, values)}
        >
            <Typography variant="body1">
                Please choose which settings from the backup to import.
            </Typography>
            <ListGrid isMobile={isMobile}>
                {importOptions.map(([key, value]) => (
                    <FieldCheckbox
                        name={`importOptions.${key}`}
                        key={key}
                        label={value}
                        CheckboxProps={{
                            sx: checkboxStyles,
                        }}
                    />
                ))}
            </ListGrid>
        </FormModal>
    );
};

interface ImportBackupModalProps {
    readonly onClose: () => void;
    readonly importOptions: [string, string][];
    readonly backupJson: Record<string, unknown>;
}
