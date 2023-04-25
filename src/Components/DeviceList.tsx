import { listen } from "@tauri-apps/api/event";
import React, { useEffect, useState } from "react";
import Device from "./Device";
import { Alert, Flex, ScrollArea, Title } from "@mantine/core";
import { invoke } from "@tauri-apps/api";
import { IconAlertCircle } from "@tabler/icons-react";

const DeviceList = () => {
    const [devices, setDevices] = useState<Device[]>([]);

    useEffect(() => {
        invoke<Array<Device>>("get_devices").then((devices) => {
            console.log("device get", devices);
            setDevices(devices);
        });
        let unlisten = listen<Array<Device>>("devices", (payload) => {
            console.log("devices", payload.payload);
            setDevices(payload.payload);
        });

        return () => {
            unlisten.then((f) => f());
        };
    }, []);
    return (
        <ScrollArea w={"100%"} h={350}>
            {/* ... content */}

            <Flex w={'100%'} direction="column" gap="sm">
                {devices
                    ?.filter((device) => device.name.includes("Bio"))
                    ?.map((device) => (
                        <Device key={device.id} device={device} />
                    ))}
            </Flex>
        </ScrollArea>
    );
};

export default DeviceList;
