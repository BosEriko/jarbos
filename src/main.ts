import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

import { fetchCurrentStates, initAgentStore } from "./agentStore";
import { AvatarManager } from "./avatarManager";
import { AVATAR_SIZE, AVATAR_SPRITE_RATIO, BOTTOM_MARGIN, setAvatarSize } from "./config";

interface DisplaySettings {
  ducksVisible: boolean;
  namesVisible: boolean;
  avatarSize: number;
}

function applyAvatarSize(size: number): void {
  document.documentElement.style.setProperty("--avatar-size", `${size}px`);
  document.documentElement.style.setProperty(
    "--avatar-sprite-size",
    `${size * AVATAR_SPRITE_RATIO}px`,
  );
}

document.documentElement.style.setProperty(
  "--avatar-bottom-margin",
  `${BOTTOM_MARGIN}px`,
);
applyAvatarSize(AVATAR_SIZE);

window.addEventListener("DOMContentLoaded", async () => {
  const root = document.querySelector<HTMLDivElement>("#avatar-root");
  if (!root) {
    return;
  }

  await initAgentStore();

  const manager = new AvatarManager(root);
  manager.start();

  await listen<boolean>("toggle-ducks-visibility", (event) => {
    manager.setDucksHidden(!event.payload);
  });

  await listen<boolean>("toggle-ducks-names", (event) => {
    manager.setNamesHidden(!event.payload);
  });

  await listen<number>("resize-ducks", (event) => {
    const size = setAvatarSize(event.payload);
    applyAvatarSize(size);
    manager.setAvatarSize(size);
  });

  const displaySettings = await invoke<DisplaySettings>("get_display_settings");
  manager.setDucksHidden(!displaySettings.ducksVisible);
  manager.setNamesHidden(!displaySettings.namesVisible);
  const size = setAvatarSize(displaySettings.avatarSize);
  applyAvatarSize(size);
  manager.setAvatarSize(size);

  const currentStates = await fetchCurrentStates();
  for (const [instanceId, { agentId, state }] of Object.entries(currentStates)) {
    manager.applyState(instanceId, agentId, state);
  }
});
