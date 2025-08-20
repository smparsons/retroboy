import { createTheme } from "@mui/material";

const smallerLineHeight = {
    styleOverrides: {
        root: {
            lineHeight: 1.0,
        },
    },
};

const theme = createTheme({
    palette: {
        mode: "dark",
        error: {
            main: "#ff7074",
        },
        primary: {
            main: "#79f6bf",
            light: "#93f7cb",
            dark: "#54ac85",
        },
        secondary: {
            main: "#e540bc",
            light: "#ea66c9",
            dark: "#a02c83",
        },
        background: {
            default: "#5e5a68",
            paper: "#7e7b86",
        },
    },
    spacing: 0.5,
    typography: {
        fontFamily: "ByteBounce, Arial",
        fontSize: 20,
    },
    components: {
        MuiButton: smallerLineHeight,
        MuiToggleButton: smallerLineHeight,
        MuiTouchRipple: smallerLineHeight,
        MuiTypography: {
            styleOverrides: {
                root: {
                    lineHeight: 1,
                },
            },
        },
        MuiCssBaseline: {
            styleOverrides: {
                "#root": {
                    height: "100%",
                },
                body: {
                    lineHeight: 1.25,
                    height: "100%",
                },
                html: {
                    height: "100%",
                },
                "@font-face": {
                    fontFamily: "ByteBounce",
                    fontStyle: "normal",
                    fontDisplay: "swap",
                    src: `local('ByteBounce'), local('ByteBounce-Regular'), url('/retroboy/ByteBounce.ttf') format('truetype')`,
                },
            },
        },
    },
});

export default theme;
