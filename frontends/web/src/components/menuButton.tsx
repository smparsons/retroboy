import {
    Divider,
    List,
    ListItem,
    ListItemIcon,
    ListItemText,
} from "@mui/material";
import Button, { ButtonProps } from "@mui/material/Button";
import Menu from "@mui/material/Menu";
import MenuItem from "@mui/material/MenuItem";
import { Fragment, useState } from "react";

import { CssGrid, GapSize, Orientation } from "./cssGrid";
import { Modal } from "./modal";

import { useIsMobile } from "../hooks/useResponsiveBreakpoint";
import { useTopLevelRenderer } from "../hooks/useTopLevelRenderer";

const MobileMenu = ({
    menuItems,
    title,
    onClose,
}: MobileMenuProps): JSX.Element => {
    return (
        <Modal heading={title} open={true} onClose={onClose}>
            <CssGrid gap={GapSize.large} orientation={Orientation.vertical}>
                <List>
                    {menuItems.map(({ action, icon, display, key }, index) => (
                        <Fragment key={key ?? index}>
                            <ListItem
                                onClick={() => {
                                    action();
                                    onClose();
                                }}
                            >
                                {icon && <ListItemIcon>{icon}</ListItemIcon>}
                                <ListItemText>{display}</ListItemText>
                            </ListItem>
                            <Divider component="li" />
                        </Fragment>
                    ))}
                </List>
            </CssGrid>
        </Modal>
    );
};

export const mobileMenuKey = "mobile-menu";

export const MenuButton = ({
    menuItems,
    withMobileMenu,
    mobileMenuTitle,
    ...buttonProps
}: MenuButtonProps): JSX.Element => {
    const [anchorEl, setAnchorEl] = useState(null as HTMLElement | null);

    const { displayTopLevelComponent, removeTopLevelComponent } =
        useTopLevelRenderer();

    const isMobile = useIsMobile();

    const handleClick = (event: React.MouseEvent<HTMLButtonElement>): void => {
        if (withMobileMenu && isMobile) {
            openMobileMenu();
        } else {
            setAnchorEl(event.currentTarget);
        }
    };

    const handleClose = (): void => {
        setAnchorEl(null);
    };

    const asMenuItemElement = (
        { action, icon, display, key }: MenuItemInfo,
        index: number,
    ): JSX.Element => (
        <MenuItem
            key={key ?? index}
            onClick={() => {
                action();
                handleClose();
            }}
        >
            {icon && <ListItemIcon>{icon}</ListItemIcon>}
            <ListItemText>{display}</ListItemText>
        </MenuItem>
    );

    const openMobileMenu = (): void => {
        displayTopLevelComponent(
            mobileMenuKey,
            <MobileMenu
                menuItems={menuItems}
                title={mobileMenuTitle ?? "Menu"}
                onClose={() => removeTopLevelComponent(mobileMenuKey)}
            />,
        );
    };

    return (
        <>
            <Button onClick={handleClick} type="button" {...buttonProps} />
            <Menu
                anchorEl={anchorEl}
                keepMounted
                open={!!anchorEl}
                onClose={handleClose}
                anchorOrigin={{ vertical: "bottom", horizontal: "left" }}
            >
                {menuItems.map(asMenuItemElement)}
            </Menu>
        </>
    );
};

export interface MenuItemInfo {
    display: React.ReactNode;
    icon?: React.ReactNode;
    action: () => void;
    key?: string;
}

interface MobileMenuProps {
    menuItems: MenuItemInfo[];
    title: string;
    onClose: () => void;
}

interface MenuButtonProps extends ButtonProps {
    menuItems: MenuItemInfo[];
    withMobileMenu?: boolean;
    mobileMenuTitle?: string;
}
