import { Component, Fragment } from "react";

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
                return (<div>
                    { tr("No results") /* TODO translate */ }
                </div>);
            } else {
                // TODO
                return storeItems.map((meta: backend.StoreMetadata) => {
                    <div>
                        <div> { meta.name } </div>
                        <div> { tr("Created by") /* TODO translate */} { meta.steam_username } </div>
                        <div> { meta.tags.map((tag: string) => <span>{tag}</span>) } </div>
                        Hey NG you should finish this page
                    </div>
                });
            }

        } else {
            backend.log(backend.LogLevel.Warn, "Store failed to load; got null from cache");
            // store did not pre-load when the game started
            return (<Fragment>
                { tr("Store failed to load") /* TODO translate */ }
            </Fragment>);
        }
    }
}
