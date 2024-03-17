import {
  ButtonItem,
  definePlugin,
  DialogButton,
  PanelSection,
  PanelSectionRow,
  ServerAPI,
  staticClasses,
  ToggleField,
  Field,
  Dropdown,
  SingleDropdownOption,
  Navigation,
  Focusable,
  Spinner,
  showModal,
  QuickAccessTab,
  ShowModalResult,
} from "decky-frontend-lib";
import { VFC, useState } from "react";
import { GiDrill, GiFireExtinguisher, GiFireBomb, GiMineExplosion } from "react-icons/gi";
import { HiRefresh, HiTrash, HiPlus, HiUpload } from "react-icons/hi";
import { TbWorldPlus } from "react-icons/tb";

//import * as python from "./python";
import * as backend from "./backend";
import { tr } from "usdpl-front";
import {
  BACKEND_INFO,
  DRIVER_INFO,

  LIMITS_INFO,

  CURRENT_BATT,
  CHARGE_RATE_BATT,
  CHARGE_MODE_BATT,
  CHARGE_LIMIT_BATT,
  CHARGE_NOW_BATT,
  CHARGE_FULL_BATT,
  CHARGE_DESIGN_BATT,
  CHARGE_POWER_BATT,

  ONLINE_CPUS,
  ONLINE_STATUS_CPUS,
  SMT_CPU,
  CLOCK_MIN_CPU,
  CLOCK_MAX_CPU,
  CLOCK_MIN_MAX_CPU,
  GOVERNOR_CPU,

  FAST_PPT_GPU,
  SLOW_PPT_GPU,
  CLOCK_MIN_GPU,
  CLOCK_MAX_GPU,
  SLOW_MEMORY_GPU,

  PERSISTENT_GEN,
  NAME_GEN,
  PATH_GEN,
  VARIANTS_GEN,
  CURRENT_VARIANT_GEN,

  MESSAGE_LIST,

  INTERNAL_STEAM_ID,
  INTERNAL_STEAM_USERNAME,

  STORE_RESULTS,
  STORE_RESULTS_URI,
  STORE_MAIN_APP_ID,

  PERIODICAL_BACKEND_PERIOD,
  AUTOMATIC_REAPPLY_WAIT,
} from "./consts";
import { set_value, get_value } from "usdpl-front";
import { Debug } from "./components/debug";
import { Gpu } from "./components/gpu";
import { Battery } from "./components/battery";
import { Cpus } from "./components/cpus";
import { DevMessages } from "./components/message";

import { TextFieldModal } from "./components/text_field_modal";

import { StoreResultsPage } from "./store/page";

var periodicHook: NodeJS.Timeout | null = null;
var lifetimeHook: any = null;
var startHook: any = null;
var endHook: any = null;
var userHook: any = null;
var usdplReady = false;
var isVariantLoading = false;

var tryNotifyProfileChange = function() {};

type MinMax = {
  min: number | null;
  max: number | null;
}

function countCpus(statii: boolean[]): number {
  let count = 0;
  for (let i = 0; i < statii.length; i++) {
    if (statii[i]) {
      count += 1;
    }
  }
  return count;
}

function syncPlebClockToAdvanced() {
  const cpuCount = (get_value(LIMITS_INFO) as backend.SettingsLimits).cpu.count;
  const minClock = get_value(CLOCK_MIN_CPU);
  const maxClock = get_value(CLOCK_MAX_CPU);
  let clockArr = [];
  for (let i = 0; i < cpuCount; i++) {
    clockArr.push({
      min: minClock,
      max: maxClock,
    } as MinMax);
  }
  set_value(CLOCK_MIN_MAX_CPU, clockArr);
}

const reload = function() {
  if (!usdplReady) {return;}

  backend.resolve(backend.getLimits(), (limits) => {
    set_value(LIMITS_INFO, limits);
    console.debug("POWERTOOLS: got limits ", limits);
  });

  if (!get_value(STORE_RESULTS)) {
    backend.resolve(backend.searchStoreByAppId(STORE_MAIN_APP_ID), (results) => set_value(STORE_RESULTS, results));
  }

  backend.resolve(backend.getBatteryCurrent(), (rate: number) => { set_value(CURRENT_BATT, rate) });
  backend.resolve_nullable(backend.getBatteryChargeRate(), (rate: number | null) => { set_value(CHARGE_RATE_BATT, rate) });
  backend.resolve_nullable(backend.getBatteryChargeMode(), (mode: string | null) => { set_value(CHARGE_MODE_BATT, mode) });
  backend.resolve_nullable(backend.getBatteryChargeLimit(), (limit: number | null) => { set_value(CHARGE_LIMIT_BATT, limit) });
  backend.resolve(backend.getBatteryChargeNow(), (rate: number) => { set_value(CHARGE_NOW_BATT, rate) });
  backend.resolve(backend.getBatteryChargeFull(), (rate: number) => { set_value(CHARGE_FULL_BATT, rate) });
  backend.resolve(backend.getBatteryChargeDesign(), (rate: number) => { set_value(CHARGE_DESIGN_BATT, rate) });
  backend.resolve(backend.getBatteryChargePower(), (rate: number) => { set_value(CHARGE_POWER_BATT, rate) });

  //backend.resolve(backend.getCpuCount(), (count: number) => { set_value(TOTAL_CPUS, count)});
  backend.resolve(backend.getCpusOnline(), (statii: boolean[]) => {
    set_value(ONLINE_STATUS_CPUS, statii);
    const count = countCpus(statii);
    set_value(ONLINE_CPUS, count);
    //set_value(SMT_CPU, statii.length > 3 && statii[0] == statii[1] && statii[2] == statii[3]);
  });
  backend.resolve(backend.getCpuSmt(), (smt: boolean) => {
    set_value(SMT_CPU, smt);
  });
  backend.resolve(backend.getCpuClockLimits(0), (limits: number[]) => {
    set_value(CLOCK_MIN_CPU, limits[0]);
    set_value(CLOCK_MAX_CPU, limits[1]);
    syncPlebClockToAdvanced();
  });
  backend.resolve(backend.getCpusGovernor(), (governors: string[]) => {
    set_value(GOVERNOR_CPU, governors);
    backend.log(backend.LogLevel.Info, "POWERTOOLS: Governors from backend " + governors.toString());
  });

  backend.resolve(backend.getGpuPpt(), (ppts: number[]) => {
    set_value(FAST_PPT_GPU, ppts[0]);
    set_value(SLOW_PPT_GPU, ppts[1]);
  });
  backend.resolve(backend.getGpuClockLimits(), (limits: number[]) => {
    set_value(CLOCK_MIN_GPU, limits[0]);
    set_value(CLOCK_MAX_GPU, limits[1]);
  });
  backend.resolve_nullable(backend.getGpuSlowMemory(), (status: number | null) => { set_value(SLOW_MEMORY_GPU, status) });

  backend.resolve(backend.getGeneralPersistent(), (value: boolean) => { set_value(PERSISTENT_GEN, value) });
  backend.resolve(backend.getGeneralSettingsName(), (name: string) => { set_value(NAME_GEN, name) });
  backend.resolve(backend.getGeneralSettingsPath(), (path: string) => { set_value(PATH_GEN, path) });
  backend.resolve(backend.getAllSettingVariants(), (variants: backend.VariantInfo[]) => { set_value(VARIANTS_GEN, variants) });
  backend.resolve(backend.getCurrentSettingVariant(), (variant: backend.VariantInfo) => { set_value(CURRENT_VARIANT_GEN, variant) });

  backend.resolve(backend.getInfo(), (info: string) => { set_value(BACKEND_INFO, info) });
  backend.resolve(backend.getDriverProviderName("gpu"), (driver: string) => { set_value(DRIVER_INFO, driver) });

  backend.resolve(backend.getMessages(null), (messages: backend.Message[]) => { set_value(MESSAGE_LIST, messages) });
};

const clearHooks = function() {
  clearInterval(periodicHook!);
  periodicHook = null;
  lifetimeHook?.unregister();
  startHook?.unregister();
  endHook?.unregister();
  userHook?.unregister();

  backend.log(backend.LogLevel.Info, "Unregistered PowerTools callbacks, so long and thanks for all the fish.");
};

const registerCallbacks = function(autoclear: boolean) {
  if (autoclear) {
    clearHooks();
  }
  // register Steam callbacks
  //@ts-ignore
  lifetimeHook = SteamClient.GameSessions.RegisterForAppLifetimeNotifications((update) => {
      backend.log(backend.LogLevel.Info, "RegisterForAppLifetimeNotifications callback(" + JSON.stringify(update, null, 2) + ")");
      if (update.bRunning) {
          //backend.log(backend.LogLevel.Debug, "AppID " + update.unAppID.toString() + " is now running");
      } else {
          //backend.log(backend.LogLevel.Debug, "AppID " + update.unAppID.toString() + " is no longer running");
          backend.resolve(backend.loadGeneralDefaultSettings(), (ok: boolean) => {
              backend.log(backend.LogLevel.Debug, "Loading default settings ok? " + ok);
              reload();
              backend.resolve(backend.waitForComplete(), (_) => {
                backend.log(backend.LogLevel.Debug, "Trying to tell UI to re-render due to game exit");
                tryNotifyProfileChange();
              });
            }
          );
          backend.resolve(
            backend.searchStoreByAppId(STORE_MAIN_APP_ID),
            (results: backend.StoreMetadata[]) => {
              set_value(STORE_RESULTS, results);
            }
          );
      }
  });
  //@ts-ignore
  startHook = SteamClient.Apps.RegisterForGameActionStart((actionType, id) => {
      //@ts-ignore
      let gameInfo: any = appStore.GetAppOverviewByGameID(id);
      let appId: string = gameInfo.appid.toString();

      backend.log(backend.LogLevel.Info, "RegisterForGameActionStart callback(" + actionType + ", " + id + ")");
      backend.resolve(
        backend.loadGeneralSettings(appId, gameInfo.display_name, "0", undefined),
        (ok: boolean) => {
          backend.log(backend.LogLevel.Debug, "Loading settings ok? " + ok);
          reload();
          backend.resolve(backend.waitForComplete(), (_) => {
            backend.log(backend.LogLevel.Debug, "Trying to tell UI to re-render due to new game launch");
            tryNotifyProfileChange();
          });
        }
      );
      backend.resolve(
        backend.searchStoreByAppId(appId),
        (results: backend.StoreMetadata[]) => {
          set_value(STORE_RESULTS, results);
        }
      );
  });

  // this fires immediately, so let's ignore that callback
  let hasFiredImmediately = false;
  //@ts-ignore
  endHook = SteamClient.Apps.RegisterForGameActionEnd((actionType) => {
      if (!hasFiredImmediately) {
        hasFiredImmediately = true;
        backend.log(backend.LogLevel.Debug, "RegisterForGameActionEnd immediately fired callback(" + actionType + ")");
        return;
      }
      backend.log(backend.LogLevel.Info, "RegisterForGameActionEnd callback(" + actionType + ")");
      setTimeout(() => backend.forceApplySettings(), AUTOMATIC_REAPPLY_WAIT);
  });

  //@ts-ignore
  userHook = SteamClient.User.RegisterForCurrentUserChanges((data) => {
      const accountName = data.strAccountName;
      const steamId = data.strSteamID;
      SteamClient.User.GetLoginUsers().then((users: any) => {
          users.forEach((user: any) => {
              if (user && user.accountName == accountName) {
                  set_value(INTERNAL_STEAM_ID, steamId);
                  set_value(INTERNAL_STEAM_USERNAME, user.personaName ? user.personaName : accountName);
              }
          });
      });
  });

  backend.log(backend.LogLevel.Debug, "Registered PowerTools callbacks, hello!");
};

// init USDPL WASM and connection to back-end
(async function(){
  await backend.initBackend();
  usdplReady = true;
  reload(); // technically this is only a load

  registerCallbacks(true);
})();

const periodicals = function() {
  backend.resolve(backend.getPeriodicals(), (periodicals) => {
    set_value(CURRENT_BATT, periodicals.battery_current);
    set_value(CHARGE_NOW_BATT, periodicals.battery_charge_now);
    set_value(CHARGE_FULL_BATT, periodicals.battery_charge_full);
    set_value(CHARGE_POWER_BATT, periodicals.battery_charge_power);

    const path = periodicals.settings_path;
    const oldValue = get_value(PATH_GEN);
    set_value(PATH_GEN, path);
    if (path != oldValue) {
      backend.log(backend.LogLevel.Debug, "Frontend values reload triggered by path change: " + oldValue + " -> " + path);
      reload();
    }
  })
};

const periodicalsSetup = function(reloadGUI: (s: string) => void) {
  if (periodicHook != null) {
    clearInterval(periodicHook);
    periodicHook = null;
  }

  periodicHook = setInterval(function() {
      periodicals();
      reloadGUI("periodic" + (new Date()).getTime().toString());
  }, PERIODICAL_BACKEND_PERIOD);
};

const Content: VFC<{ serverAPI: ServerAPI }> = ({}) => {

  const [idc, reloadGUI] = useState<any>("/shrug");
  tryNotifyProfileChange = function() { reloadGUI("ProfileChangeByNotification") };

  periodicalsSetup(reloadGUI);

  if (!usdplReady || !get_value(LIMITS_INFO)) {
    // Not translated on purpose (to avoid USDPL issues)
    return (
      <PanelSection>
        USDPL or PowerTools's backend did not start correctly!
        <ButtonItem
          layout="below"
          onClick={(_: MouseEvent) => {
            console.log("POWERTOOLS: manual reload after startup failure");
            reload();
            // try to reload GUI too
            backend.resolve(backend.waitForComplete(), (_) => {reloadGUI("LoadSystemDefaults")});
          }}
        >
        Reload
        </ButtonItem>
      </PanelSection>
    )
  }

  const variantOptions: SingleDropdownOption[] = (get_value(VARIANTS_GEN) as backend.VariantInfo[]).map((elem) => {return {
        data: elem,
        label: <span>{elem.name}</span>,
    };});

  var modalResult: ShowModalResult | undefined = undefined;

  const onNewVariantModelClosed = (name: string) => {
    if (modalResult) {
      modalResult.Close();
    }
    console.log("POWERTOOLS: variant name", name);
    isVariantLoading = true;
    backend.resolve(
      backend.loadGeneralSettingsVariant("please give me a new ID k thx bye" /* anything that cannot be parsed as a u64 will be set to u64::MAX, which will cause the back-end to auto-generate an ID */, name),
      (ok: boolean) => {
        backend.log(backend.LogLevel.Debug, "New settings variant ok? " + ok);
        reload();
        backend.resolve(backend.waitForComplete(), (_) => {
          backend.log(backend.LogLevel.Debug, "Trying to tell UI to re-render due to new settings variant");
          tryNotifyProfileChange();
        });
      }
    );
    Navigation.OpenQuickAccessMenu(QuickAccessTab.Decky);
  };

  return (
    <PanelSection>
      <DevMessages idc={idc}/>

      <Cpus idc={idc}/>

      <Gpu idc={idc}/>

      <Battery idc={idc}/>


      {/* Persistence */}
      <div className={staticClasses.PanelSectionTitle}>
        {tr("Miscellaneous")}
      </div>
      <PanelSectionRow>
        <ToggleField
          checked={get_value(PERSISTENT_GEN)}
          label={tr("Persistent Profile")}
          description={tr("Save profile and load it next time")}
          onChange={(persist: boolean) => {
            backend.log(backend.LogLevel.Debug, "Persist is now " + persist.toString());
            backend.resolve(
              backend.setGeneralPersistent(persist),
              (val: boolean) => {set_value(PERSISTENT_GEN, val)}
            );
          }}
        />
      </PanelSectionRow>
      <PanelSectionRow>
        <Field
          label={tr("Profile")}>
          {get_value(NAME_GEN)}
        </Field>
      </PanelSectionRow>
      <PanelSectionRow>
        <Field
          label={tr("Profile Variant")} // TODO translate
          >
          {(!isVariantLoading && <Dropdown
              menuLabel={tr("Profile Variant")}
              rgOptions={variantOptions}
              selectedOption={variantOptions.find((val: SingleDropdownOption, _index, _arr) => {
                backend.log(backend.LogLevel.Debug, "POWERTOOLS: looking for variant data.id " + (get_value(CURRENT_VARIANT_GEN) as backend.VariantInfo).id.toString());
                return (val.data as backend.VariantInfo).id == (get_value(CURRENT_VARIANT_GEN) as backend.VariantInfo).id;
              })}
              strDefaultLabel={(get_value(CURRENT_VARIANT_GEN) as backend.VariantInfo | undefined)?.name?? (get_value(VARIANTS_GEN) as backend.VariantInfo[])[0].name}
              onChange={(elem: SingleDropdownOption) => {
                if (elem.data.id != (get_value(CURRENT_VARIANT_GEN) as backend.VariantInfo).id) {
                  isVariantLoading = true;
                  let data = elem.data as backend.VariantInfo;
                  backend.log(backend.LogLevel.Debug, "Profile variant dropdown selected " + elem.data.id.toString());
                  //set_value(CURRENT_VARIANT_GEN, elem.data as backend.VariantInfo);
                  backend.resolve(
                    backend.loadGeneralSettingsVariant(data.id, data.name),
                    (ok: boolean) => {
                      backend.log(backend.LogLevel.Debug, "Loaded settings variant ok? " + ok);
                      reload();
                      isVariantLoading = false;
                      backend.resolve(backend.waitForComplete(), (_) => {
                        backend.log(backend.LogLevel.Debug, "Trying to tell UI to re-render due to newly-selected settings variant");
                        tryNotifyProfileChange();
                        //reloadGUI("ProfileVariantSelected");
                      });
                    }
                  );
                }
              }}
          />)}
          {(isVariantLoading && <Spinner/>)}
        </Field>
      </PanelSectionRow>
      <PanelSectionRow>
        <Focusable style={{
          alignItems: "center",
          display: "flex",
          justifyContent: "space-around",
        }}
          flow-children="horizontal"
        >
          <DialogButton
            style={{
              maxWidth: "30%",
              minWidth: "auto",
            }}
            //layout="below"
            onClick={(_: MouseEvent) => {
              backend.log(backend.LogLevel.Debug, "Creating new PowerTools settings variant");
              modalResult = showModal(<TextFieldModal label={tr("Profile Variant") /* TODO translate */} placeholder={tr("Variant name") /* TODO translate */} onClosed={onNewVariantModelClosed} closeModal={() => { modalResult?.Close(); Navigation.OpenQuickAccessMenu(QuickAccessTab.Decky)}}/>, window);
            }}
          >
            <HiPlus/>
          </DialogButton>
          <DialogButton
            style={{
              maxWidth: "30%",
              minWidth: "auto",
            }}
            //layout="below"
            onClick={(_: MouseEvent) => {
              const steamId = get_value(INTERNAL_STEAM_ID);
              const steamName = get_value(INTERNAL_STEAM_USERNAME);
              if (steamId && steamName) {
                  backend.storeUpload(steamId, steamName);
              } else {
                  backend.log(backend.LogLevel.Warn, "Cannot upload with null steamID (is null: " + !steamId + ") and/or username (is null: " + !steamName + ")");
              }
            }}
          >
            <HiUpload/>
          </DialogButton>
          <DialogButton
            style={{
              maxWidth: "30%",
              minWidth: "auto",
            }}
            //layout="below"
            onClick={(_: MouseEvent) => {
              Navigation.Navigate(STORE_RESULTS_URI);
              Navigation.CloseSideMenus();
            }}
          >
            <TbWorldPlus />
          </DialogButton>
        </Focusable>
      </PanelSectionRow>

      <Debug idc={idc}/>

      <PanelSectionRow>
        <ButtonItem
          layout="below"
          onClick={(_: MouseEvent) => {
            backend.log(backend.LogLevel.Debug, "Reapplying PowerTools settings");
            backend.forceApplySettings();
          }}
        >
        <HiRefresh /> {tr("Reapply settings")}
        </ButtonItem>
      </PanelSectionRow>

      <PanelSectionRow>
        <ButtonItem
          layout="below"
          onClick={(_: MouseEvent) => {
            backend.log(backend.LogLevel.Debug, "Loading default PowerTools settings");
            backend.resolve(
              backend.setGeneralPersistent(false),
              (val: boolean) => {
                set_value(PERSISTENT_GEN, val);
                backend.resolve(backend.loadGeneralSystemSettings(), (_) => {
                  reload();
                  backend.resolve(backend.waitForComplete(), (_) => {reloadGUI("LoadSystemDefaults")});
                });
              }
            );
          }}
        >
        <HiTrash /> {tr("Defaults")}
        </ButtonItem>
      </PanelSectionRow>
    </PanelSection>
  );
};

export default definePlugin((serverApi: ServerAPI) => {
  let ico = <GiDrill />;
  let now = new Date();
  if (now.getDate() == 1 && now.getMonth() == 3) {
    ico = <span><GiFireExtinguisher /><GiFireBomb /><GiMineExplosion /></span>;
  }
  //registerCallbacks(false);
  serverApi.routerHook.addRoute(STORE_RESULTS_URI, () => <StoreResultsPage onNewVariant={tryNotifyProfileChange}/>);
  return {
    title: <div className={staticClasses.Title}>PowerTools</div>,
    content: <Content serverAPI={serverApi} />,
    icon: ico,
    onDismount() {
      tryNotifyProfileChange = function() {};
      backend.log(backend.LogLevel.Debug, "PowerTools shutting down");
      clearHooks();
      serverApi.routerHook.removeRoute(STORE_RESULTS_URI);
    },
  };
});
