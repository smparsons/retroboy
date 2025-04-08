import AddIcon from "@mui/icons-material/Add";
import MoreVertIcon from "@mui/icons-material/MoreVert";
import { Button, Checkbox, styled, Typography } from "@mui/material";

import AddCheatModal from "./addCheatModal";

import { CssGrid, GapSize, Orientation, Position } from "../components/cssGrid";
import { ListGrid, ListItemGrid } from "../components/list";
import { MenuButton } from "../components/menuButton";
import { Modal, ModalGridButton } from "../components/modal";
import {
    registerGamegenieCheat,
    registerGamesharkCheat,
    unregisterCheat,
} from "../core/retroboyCore";
import { useIsMobile } from "../hooks/useResponsiveBreakpoint";
import { useSettingsStore } from "../hooks/useSettingsStore";
import { useToastRenderer } from "../hooks/useToastRenderer";
import { useTopLevelRenderer } from "../hooks/useTopLevelRenderer";
import { YesNoDialog } from "../modals/dialog";

const CheatDescriptionGrid = styled(CssGrid)`
    p,
    span {
        line-height: 0.75;
    }
`;

const deleteConfirmationModalKey = "delete-cheat-confirmation-modal";
const addCheatModalKey = "add-cheat-modal";

export const CheatsModal = ({
    gameKey,
    onClose,
}: CheatsModalProps): JSX.Element => {
    const { settings, storeSettings } = useSettingsStore();
    const toast = useToastRenderer();
    const { displayTopLevelComponent, removeTopLevelComponent } =
        useTopLevelRenderer();
    const isMobile = useIsMobile();

    const cheatsById = settings.cheats ? settings.cheats[gameKey] : {};
    const cheats = Object.values(cheatsById || {});

    const handleAddOrEditCheat = (cheat: Cheat | null): void => {
        displayTopLevelComponent(
            addCheatModalKey,
            <AddCheatModal
                gameKey={gameKey}
                cheat={cheat}
                onClose={() => removeTopLevelComponent(addCheatModalKey)}
            />,
        );
    };

    const updateCheatRegistration = (
        cheat: Cheat,
        registered: boolean,
    ): string | undefined => {
        if (registered) {
            if (cheat.type === CheatType.GameShark) {
                return registerGamesharkCheat(cheat.id, cheat.code);
            } else {
                return registerGamegenieCheat(cheat.id, cheat.code);
            }
        } else {
            unregisterCheat(cheat.id);
            return undefined;
        }
    };

    const storeRegistrationUpdate = (
        cheat: Cheat,
        registered: boolean,
    ): void => {
        const error = updateCheatRegistration(cheat, registered);
        if (error) {
            toast.error(error);
        } else {
            storeSettings({
                ...settings,
                cheats: {
                    ...settings.cheats,
                    [gameKey]: {
                        ...(settings.cheats ? settings.cheats[gameKey] : {}),
                        [cheat.id]: {
                            ...cheat,
                            registered,
                        },
                    },
                },
            });
        }
    };

    const handleDeleteCheat = (cheat: Cheat): void => {
        const updatedCheats = { ...cheatsById };
        delete updatedCheats[cheat.id];

        if (cheat.registered) {
            unregisterCheat(cheat.id);
        }

        storeSettings({
            ...settings,
            cheats: {
                ...settings.cheats,
                [gameKey]: updatedCheats,
            },
        });
    };

    const handleConfirmDelete = (cheat: Cheat): void => {
        displayTopLevelComponent(
            deleteConfirmationModalKey,
            <YesNoDialog
                onClose={() =>
                    removeTopLevelComponent(deleteConfirmationModalKey)
                }
                onYes={() => {
                    handleDeleteCheat(cheat);
                    removeTopLevelComponent(deleteConfirmationModalKey);
                }}
                onNo={() => removeTopLevelComponent(deleteConfirmationModalKey)}
                message={`Are you sure you want to delete the cheat "${cheat.name}"?`}
                heading="Delete Cheat"
            />,
        );
    };

    return (
        <Modal heading="Cheats" open={true} onClose={onClose}>
            <CssGrid orientation={Orientation.vertical} gap={GapSize.large}>
                <CssGrid
                    gap={GapSize.large}
                    orientation={
                        isMobile ? Orientation.vertical : Orientation.horizontal
                    }
                    justifyContent={
                        isMobile ? Position.stretch : Position.start
                    }
                >
                    <Button
                        variant="contained"
                        color="primary"
                        startIcon={<AddIcon />}
                        onClick={() => handleAddOrEditCheat(null)}
                    >
                        Add
                    </Button>
                </CssGrid>
                {cheats.length ? (
                    <ListGrid
                        orientation={Orientation.vertical}
                        gap={GapSize.medium}
                        isMobile={isMobile}
                    >
                        {cheats.map(cheat => (
                            <ListItemGrid
                                orientation={Orientation.horizontal}
                                alignContent={Position.center}
                                alignItems={Position.center}
                                justifyContent={Position.end}
                                template="auto 1fr auto"
                                key={cheat.id}
                            >
                                <Checkbox
                                    checked={cheat.registered}
                                    onClick={event => {
                                        event.stopPropagation();
                                        storeRegistrationUpdate(
                                            cheat,
                                            !cheat.registered,
                                        );
                                    }}
                                    value={cheat.id}
                                />
                                <CheatDescriptionGrid
                                    orientation={Orientation.vertical}
                                    gap={GapSize.small}
                                    onClick={() => {
                                        storeRegistrationUpdate(
                                            cheat,
                                            !cheat.registered,
                                        );
                                    }}
                                >
                                    <Typography variant="body1">
                                        {cheat.name}
                                    </Typography>
                                    <Typography variant="caption">
                                        Code: {cheat.code}
                                    </Typography>
                                </CheatDescriptionGrid>
                                <MenuButton
                                    variant="contained"
                                    color="secondary"
                                    menuItems={[
                                        {
                                            display: "Edit",
                                            action: () =>
                                                handleAddOrEditCheat(cheat),
                                            key: "edit",
                                        },
                                        {
                                            display: "Delete",
                                            action: () =>
                                                handleConfirmDelete(cheat),
                                            key: "delete",
                                        },
                                    ]}
                                >
                                    <MoreVertIcon fontSize="small" />
                                </MenuButton>
                            </ListItemGrid>
                        ))}
                    </ListGrid>
                ) : (
                    <Typography variant="body1">
                        You currently have no cheats. Click "Add" to create a
                        new cheat.
                    </Typography>
                )}
                <ModalGridButton
                    variant="contained"
                    onClick={onClose}
                    isMobile={isMobile}
                >
                    Close
                </ModalGridButton>
            </CssGrid>
        </Modal>
    );
};

export enum CheatType {
    GameShark = "gameshark",
    GameGenie = "gamegenie",
}

export interface Cheat {
    readonly id: string;
    readonly name: string;
    readonly code: string;
    readonly type: CheatType;
    readonly registered: boolean;
}

interface CheatsModalProps {
    readonly gameKey: string;
    readonly onClose: () => void;
}
