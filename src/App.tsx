import { useEffect, useState } from "react";
import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/tauri";
import "./App.css";
import { emit, listen } from "@tauri-apps/api/event";
import Device from "./Components/Device";
import { Button, Flex, Title, Text, Card, Paper, Tooltip } from "@mantine/core";
import Home from "./routes/Home";
import { appWindow } from "@tauri-apps/api/window";
import { IconWindowMinimize, IconX } from "@tabler/icons-react";
import { Outlet } from "react-router-dom";

function App() {
    return (
        <Flex direction={"column"} gap={"sm"}>
            <div data-tauri-drag-region className="titlebar">
                <div
                    onClick={(e) => {
                        e.preventDefault();
                        appWindow.minimize();
                    }}
                    className="titlebar-button"
                    id="titlebar-minimize"
                >
                    <Tooltip  offset={5} openDelay={500}  withArrow label="Minimize">
                        <IconWindowMinimize size={"1.2rem"} />
                    </Tooltip>
                </div>
                <div
                    onClick={(e) => {
                        e.preventDefault();
                        appWindow.close();
                    }}
                    className="titlebar-button"
                    id="titlebar-close"
                >
                    <Tooltip offset={5} openDelay={500}  withArrow label="Close">
                        <IconX  size={"1.2rem"} />
                    </Tooltip>
                </div>
            </div>
            <Outlet />
        </Flex>
    );
}

export default App;
