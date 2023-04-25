import { create } from "zustand";

interface BleState {
    device: Device | undefined;
    setDevice: (device: Device) => void;
}

const useStore = create<BleState>()((set) => ({
    device: undefined,
    setDevice: (id) => set((state) => ({ device: id })),
}));

export default useStore;
