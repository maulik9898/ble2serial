import { useEffect, useState } from "react";
import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/tauri";
import { emit, listen } from "@tauri-apps/api/event";
import { Button, Flex, Title, Text, Card, Paper, Alert } from "@mantine/core";
import DeviceList from "../Components/DeviceList";
import useStore from "../store";
import { useNavigate } from "react-router-dom";
import { notifications } from "@mantine/notifications";
import { IconAlertCircle, IconExclamationCircle } from "@tabler/icons-react";
import { trace, info, error, attachConsole } from "tauri-plugin-log-api";

function Home() {
    const device = useStore((state) => state.device);
    const [loading, setLoading] = useState(false);
    const [bleon, setBleon] = useState(true);
    const [retryLoading, setRetryLoading] = useState(false);
    const navigate = useNavigate();
    async function start_scan() {
        setRetryLoading(true);
        console.log("start_scan");
        // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
        try {
            await invoke("start_scan");
            setBleon(true);
        } catch (err) {
            setBleon(false);
            error(err as string);
        } finally {
            setRetryLoading(false);
        }
    }

    async function connect() {
        setLoading(true);
        console.log("connect");
        if (device) {
            try {
                await invoke("connect", { id: device.id });
            } catch (e) {
                error(e as string);
                setLoading(false);
                notifications.show({
                    title: "Connection Error",
                    message: e as string,
                    withBorder: true,
                    color: "red",
                });
            }
        }
    }

    useEffect(() => {
        start_scan();
    }, []);

    useEffect(() => {
        let unlisten = listen("connected", (event) => {
            info(`connected : {event}`);
            setLoading(false);
            navigate("/connected");
        });
        return () => {
            unlisten.then((unlisten) => unlisten());
        };
    }, []);

    useEffect(() => {
        let unlisten = listen<string>("error", (event) => {
            error(`error : {event}  `);
            setLoading(false);
            notifications.show({
                title: "Connection Error",
                message: event.payload,
                withBorder: true,
                color: "red",
            });
        });
        return () => {
            unlisten.then((unlisten) => unlisten());
        };
    }, []);

    return (
        <Flex w={"100%"} direction={"column"} align={"center"}>
            <Flex
                gap={"md"}
                direction={"column"}
                align={"center"}
                w={"90%"}
                justify={"center"}
            >
                <Title color="dimmed" mb={"sm"}>
                    BS BLE2COM
                </Title>
                {bleon ? (
                    <>
                        <DeviceList />
                        <Button
                            loading={loading}
                            disabled={device === undefined}
                            color="green"
                            w={"100%"}
                            variant="outline"
                            onClick={connect}
                        >
                            Connect
                        </Button>
                    </>
                ) : (
                    <Flex direction={"column"} gap={"md"} m={"lg"} p={"md"}>
                        <Alert
                            icon={<IconAlertCircle size="1rem" />}
                            title="Bummer!"
                            color="red"
                            variant="outline"
                        >
                            Bluetooth is not turned on. Please turn on the
                            bluetooth and try again.
                        </Alert>
                        <Button
                            loading={retryLoading}
                            color="orange"
                            variant="outline"
                            fullWidth
                            onClick={start_scan}
                        >
                            Retry
                        </Button>
                    </Flex>
                )}
            </Flex>
        </Flex>
    );
}

export default Home;
