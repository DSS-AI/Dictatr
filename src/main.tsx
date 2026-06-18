import React from "react";
import ReactDOM from "react-dom/client";
import { getCurrentWindow } from "@tauri-apps/api/window";
import App from "./App";
import PromptPicker from "./PromptPicker";
import "./index.css";

// Both the settings window and the Prompt-Manager popup load this same bundle.
// The window label (read synchronously, no IPC) decides which view renders.
const isPicker = getCurrentWindow().label === "prompt-picker";

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    {isPicker ? <PromptPicker /> : <App />}
  </React.StrictMode>,
);
