import { fetchCurrentStates, initAgentStore } from "./agentStore";
import { AvatarManager } from "./avatarManager";
import { AVATAR_SIZE, BOTTOM_MARGIN } from "./config";

document.documentElement.style.setProperty(
  "--avatar-bottom-margin",
  `${BOTTOM_MARGIN}px`,
);
document.documentElement.style.setProperty("--avatar-size", `${AVATAR_SIZE}px`);

window.addEventListener("DOMContentLoaded", async () => {
  const root = document.querySelector<HTMLDivElement>("#avatar-root");
  if (!root) {
    return;
  }

  await initAgentStore();

  const manager = new AvatarManager(root);
  manager.start();

  const currentStates = await fetchCurrentStates();
  for (const [id, state] of Object.entries(currentStates)) {
    manager.applyState(id, state);
  }
});
