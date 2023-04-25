import { Card, Paper, Text } from "@mantine/core";
import React from "react";
import useStore from "../store";

const Device = ({ device }: { device: Device }) => {
    const [d, setDevice] = useStore((state) => [state.device, state.setDevice]);
    return (
        <Card
            m={'auto'}
            w={'100%'}
            sx={(theme) => {
                return {
                    backgroundColor:
                        d?.id === device.id
                            ? theme.colors.dark[9]
                            : theme.colorScheme === "dark"
                            ? theme.colors.dark[8]
                            : theme.colors.gray[0],
                    color:
                        theme.colorScheme === "dark"
                            ? theme.colors.dark[0]
                            : theme.colors.dark[9],

                    border: `1px solid ${
                        d?.id === device.id
                            ? "#51cf66"
                            : theme.colorScheme === "dark"
                            ? theme.colors.dark[8]
                            : theme.colors.gray[2]
                    }`,

                    "&:hover": {
                        cursor: "pointer",
                        backgroundColor:
                            theme.colorScheme === "dark"
                                ? theme.colors.dark[9]
                                : theme.colors.gray[1],
                    },
                };
            }}
            p="xs"
            shadow="xs"
            onClick={() => {
                setDevice(device);
            }}
        >
            <Text size="md" weight={500}>
                {device.name}
            </Text>
            <Text size="sm" color="dimmed">
                {device.id}
            </Text>
        </Card>
    );
};

export default Device;
