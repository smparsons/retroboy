import { CssBaseline, ThemeProvider } from "@mui/material";

import registerCartridgeEffects from "./cartridgeEffects";
import WasmLoader from "./components/wasmLoader";
import { ResponsiveBreakpointProvider } from "./hooks/useResponsiveBreakpoint";
import { SettingsStoreProvider } from "./hooks/useSettingsStore";
import { TopLevelRendererProvider } from "./hooks/useTopLevelRenderer";
import Interface from "./interface";
import theme from "./theme";

registerCartridgeEffects();

const App = (): JSX.Element => (
    <ThemeProvider theme={theme}>
        <CssBaseline />
        <SettingsStoreProvider>
            <ResponsiveBreakpointProvider>
                <TopLevelRendererProvider>
                    <WasmLoader>
                        <Interface />
                    </WasmLoader>
                </TopLevelRendererProvider>
            </ResponsiveBreakpointProvider>
        </SettingsStoreProvider>
    </ThemeProvider>
);

export default App;
