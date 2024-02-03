import { Component } from "react";
import { HiDownload } from "react-icons/hi";

import { PanelSectionRow, PanelSection, Focusable, staticClasses, DialogButton } from "decky-frontend-lib";

import * as backend from "../backend";
import { tr } from "usdpl-front";
import { get_value} from "usdpl-front";

import {
    STORE_RESULTS,
} from "../consts";

export class StoreResultsPage extends Component {
    constructor() {
        super({});
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
                return (<PanelSection>
                    { tr("No results") /* TODO translate */ }
                </PanelSection>);
            } else {
                // TODO
                return (<PanelSection>
                    {
                        storeItems.map((meta: backend.StoreMetadata) => {
                            <PanelSectionRow>
                                <Focusable style={{
                                    display: "flex",
                                    flexDirection: "row",
                                    borderRadius: "2px",
                                }}>
                                    <DialogButton
                                        //layout="below"
                                        style={{
                                        width: "56px",
                                        display: "flex",
                                        flexShrink: "0",
                                        alignSelf: "center",
                                        }}
                                        onClick={(_: MouseEvent) => {
                                            backend.log(backend.LogLevel.Info, "Downloading settings " + meta.name + " (" + meta.id + ")");
                                            backend.storeDownloadById(meta.id);
                                        }}
                                    >
                                        { /* TODO make this responsive when clicked */}
                                        <HiDownload/>
                                    </DialogButton>
                                    <div style={{
                                        flexGrow: "1",
                                        display: "flex",
                                        flexDirection: "column",
                                        minWidth: "0",
                                        marginBottom: "2px",
                                    }}
                                    className={staticClasses.Text}>
                                        <div style={{
                                            display: "flex",
                                            flexDirection: "row",
                                            minWidth: "0",
                                            fontSize: "16px",
                                        }}
                                        className={staticClasses.Text}>
                                            { meta.name }
                                        </div>
                                        <div style={{
                                            display: "flex",
                                            flexDirection: "row",
                                            minWidth: "0",
                                            fontSize: "12px",
                                        }}
                                        className={staticClasses.Text}>
                                            { tr("Created by") /* TODO translate */} { meta.steam_username }
                                        </div>
                                        <div>
                                            { meta.tags.map((tag: string) => (<span style={{
                                                    borderRadius: "10px",
                                                }}>
                                                    {tag}
                                                </span>)
                                            ) }
                                        </div>
                                    </div>
                                </Focusable>
                            </PanelSectionRow>
                        })
                    }
                    </PanelSection>);
            }

        } else {
            backend.log(backend.LogLevel.Warn, "Store failed to load; got null from cache");
            // store did not pre-load when the game started
            return (<PanelSection>
                { tr("Store failed to load") /* TODO translate */ }
            </PanelSection>);
        }
    }
}
