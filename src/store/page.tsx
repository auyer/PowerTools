import { Component } from "react";
import { HiDownload } from "react-icons/hi";

import { PanelSectionRow, Focusable, staticClasses, DialogButton } from "decky-frontend-lib";

import * as backend from "../backend";
import { tr } from "usdpl-front";
import { get_value, set_value } from "usdpl-front";

import {
    STORE_RESULTS,
    VARIANTS_GEN,
} from "../consts";

export class StoreResultsPage extends Component<{onNewVariant: () => void}> {
    constructor(props: {onNewVariant: () => void}) {
        super(props);
        this.state = {
            reloadThingy: "/shrug",
        };
    }

    render() {
        const storeItems = get_value(STORE_RESULTS) as backend.StoreMetadata[] | undefined;
        console.log("POWERTOOLS: Rendering store results", storeItems);
        if (storeItems) {
            if (storeItems.length == 0) {
                backend.log(backend.LogLevel.Warn, "No store results; got array with length 0 from cache");
                return (<Focusable
                    style={{
                        display: "flex",
                        flexWrap: "wrap",
                        justifyContent: "center",
                        rowGap: "5px",
                        columnGap: "5px",
                        maxWidth: "100%",
                        margin: "auto",
                    }}
                >
                    { tr("No results") /* TODO translate */ }
                </Focusable>);
            } else {
                // TODO
                return (<Focusable
                    style={{
                        display: "flex",
                        flexWrap: "wrap",
                        justifyContent: "center",
                        rowGap: "5px",
                        columnGap: "5px",
                        maxWidth: "100%",
                        margin: "2em 0.5em",
                    }}
                >
                    {
                        storeItems.map((meta: backend.StoreMetadata) => (<PanelSectionRow>
                                <Focusable style={{
                                    display: "flex",
                                    flexDirection: "row",
                                    borderRadius: "1em",
                                    maxWidth: "100%",
                                    padding: "1em",
                                    //borderColor: "#93b3c8", // light blue bg colour
                                    //borderWidth: "0.1em",
                                    //borderStyle: "solid",
                                    margin: "0.5em",
                                }}>
                                    <div style={{
                                        flexGrow: "1",
                                        display: "flex",
                                        flexDirection: "column",
                                        minWidth: "0",
                                        marginBottom: "2px",
                                        padding: "0.25em",
                                    }}
                                    className={staticClasses.Text}>
                                        <div style={{
                                            display: "flex",
                                            flexDirection: "row",
                                            minWidth: "0",
                                            fontSize: "24px",
                                            padding: "0.25em",
                                        }}
                                        className={staticClasses.Text}>
                                            { meta.name }
                                        </div>
                                        <div style={{
                                            display: "flex",
                                            flexDirection: "row",
                                            minWidth: "0",
                                            fontSize: "20px",
                                            padding: "0.25em",
                                        }}
                                        className={staticClasses.Text}>
                                            { tr("Created by") /* TODO translate */} { meta.steam_username }
                                        </div>
                                        <div style={{
                                            display: "flex",
                                            flexDirection: "row",
                                            minWidth: "0",
                                            fontSize: "16px",
                                            padding: "0.25em",
                                        }}>
                                            { meta.tags.map((tag: string) => (<span style={{
                                                    borderRadius: "1em",
                                                    borderColor: "#dbe2e6", // main text colour
                                                    backgroundColor: "#dbe2e6",
                                                    color: "#1f2126", // main top image bg colour
                                                    padding: "0.25em",
                                                    margin: "0.25em",
                                                    display: "flex",
                                                    minWidth: "0",
                                                    justifyContent: "center",
                                                    fontSize: "12px",
                                                    flexGrow: "1",
                                                }}>
                                                    {tag}
                                                </span>)
                                            ) }
                                        </div>
                                    </div>
                                    <DialogButton
                                        //layout="below"
                                        style={{
                                        minWidth: "56px",
                                        maxWidth: "56px",
                                        display: "flex",
                                        flexShrink: "0",
                                        alignSelf: "center",
                                        padding: "1em",
                                        margin: "0.5em",
                                        }}
                                        onClick={(_: MouseEvent) => {
                                            backend.log(backend.LogLevel.Info, "Downloading settings " + meta.name + " (" + meta.id + ")");
                                            backend.resolve(backend.storeDownloadById(meta.id),
                                                (variants: backend.VariantInfo[]) => {
                                                    set_value(VARIANTS_GEN, variants)
                                                    this.props.onNewVariant();
                                                }
                                            );
                                        }}
                                    >
                                        { /* TODO make this responsive when clicked */}
                                        <HiDownload style={{
                                            height: "100%",
                                            width: "100%",
                                        }}/>
                                    </DialogButton>
                                </Focusable>
                            </PanelSectionRow>))
                    }
                    </Focusable>);
            }

        } else {
            backend.log(backend.LogLevel.Warn, "Store failed to load; got null from cache");
            // store did not pre-load when the game started
            return (<Focusable
                    style={{
                        display: "flex",
                        flexWrap: "wrap",
                        justifyContent: "center",
                        rowGap: "5px",
                        columnGap: "5px",
                        maxWidth: "100%",
                        margin: "auto",
                    }}
                >
                { tr("Store failed to load") /* TODO translate */ }
            </Focusable>);
        }
    }
}
