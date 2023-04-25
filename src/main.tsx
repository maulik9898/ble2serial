import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./styles.css";
import { MantineProvider } from "@mantine/core";
import { createBrowserRouter, RouterProvider } from "react-router-dom";
import Connected from "./routes/Connected";
import DeviceList from "./Components/DeviceList";
import Home from "./routes/Home";
import { Notifications } from "@mantine/notifications";

const router = createBrowserRouter([
    {
        path: "/",
        element: <App />,
        children: [
            {
                path: "",
                element: <Home />,
            },
            {
                path: "connected",
                element: <Connected />,
            },
        ],
    },
]);

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
    <React.StrictMode>
        <MantineProvider
            withGlobalStyles
            withNormalizeCSS
            theme={{ colorScheme: "dark" }}
        >
            <Notifications />
            <RouterProvider router={router} />
        </MantineProvider>
    </React.StrictMode>
);
