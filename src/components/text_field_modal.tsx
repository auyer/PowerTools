// Based on https://github.com/isiah-lloyd/radiyo-steam-deck/blob/main/src/TextFieldModal.tsx

import { ModalRoot, ModalRootProps, Router, TextField, Focusable, DialogButton } from 'decky-frontend-lib';
import { useEffect, useRef, useState } from 'react';
import { HiCheck, HiX } from "react-icons/hi";
type props = ModalRootProps & {
    label: string,
    placeholder: string,
    onClosed: (inputText: string) => void;
}
export const TextFieldModal = ({ closeModal, onClosed, label, placeholder }: props) => {
    const [inputText, setInputText] = useState('');
    const handleText = (e: React.ChangeEvent<HTMLInputElement>) => {
        setInputText(e.target.value);
    };
    const textField = useRef<any>();
    useEffect(() => {
        Router.CloseSideMenus();
        //This will open up the virtual keyboard
        textField.current?.element?.click();
    }, []);
    const submit = () => onClosed(inputText);
    return (
        <ModalRoot closeModal={closeModal} onEscKeypress={closeModal}>
            <form onSubmit={submit}>
                <TextField
                    //@ts-ignore
                    ref={textField}
                    focusOnMount={true}
                    label={label}
                    placeholder={placeholder}
                    onChange={handleText}
                />
                <Focusable style={{
                alignItems: "center",
                display: "flex",
                justifyContent: "space-around",
                }}
                flow-children="horizontal"
                >
                <DialogButton
                    style={{
                    maxWidth: "45%",
                    minWidth: "auto",
                    }}
                    //layout="below"
                    onClick={(_: MouseEvent) => { submit() }}
                >
                    <HiCheck/>
                </DialogButton>
                <DialogButton
                    style={{
                    maxWidth: "45%",
                    minWidth: "auto",
                    }}
                    //layout="below"
                    onClick={(_: MouseEvent) => { if (closeModal) { closeModal() } }}
                >
                    <HiX/>
                </DialogButton>
                </Focusable>
            </form>
        </ModalRoot>
    );
};
