import { Box, Button, Card, Flex, Title } from "@mantine/core";
import { invoke } from "@tauri-apps/api";
import React, { useEffect, useState } from "react";
import useStore from "../store";
import { listen } from "@tauri-apps/api/event";
import { useNavigate } from "react-router-dom";
import Device from "../Components/Device";

const Connected = () => {
    const device = useStore((state) => state.device);
    const [loading, setLoading] = useState(false);
    const [port, setPort] = useState<string | undefined>(undefined);
    const navigate = useNavigate();
    async function disconnect() {
        setLoading(true);
        console.log("connect");
        if (device) {
            await invoke("disconnect", { id: device.id });
        }
    }

    useEffect(() => {
        invoke<string | undefined>("get_other_port").then((port) => {
            setPort(port);
        });
    }, []);

    useEffect(() => {
        let unlisten = listen("disconnected", (event) => {
            console.log("disconnected : ", event);
            setLoading(false);
            navigate("/");
        });
        return () => {
            unlisten.then((unlisten) => unlisten());
        };
    }, []);
    useEffect(() => {
        let unlisten = listen("error", (event) => {
            console.log("error : ", event);
            setLoading(false);
            
        });
        return () => {
            unlisten.then((unlisten) => unlisten());
        };
    }, []);


    return (
        <Flex w={'90%'} m={'auto'}  align={"start"} direction={"column"} gap={"lg"}>
            <Title mx={"sm"} order={2}>
                {" "}
                Connected to{" "}
            </Title>
            <Device device={device!} />
            <Title  order={4}>
                Use below port in you app
            </Title>
            <Card mx={"auto"} w={"100%"}>
                <Title align="center">{port}</Title>
            </Card>
            <Box w={"100%"} mx={"auto"}>
                <Button
                    fullWidth
                    variant="outline"
                    color="red"
                    loading={loading}
                    onClick={disconnect}
                >
                    Disconnect
                </Button>
            </Box>
        </Flex>
    );
};

export default Connected;
