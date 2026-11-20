import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import App from "./App";
import "../design/tokens.css";
import "./styles/base.css";
import "./styles/layout.css";
import "./styles/dashboard.css";
import "./styles/module.css";
import "./styles/overlays.css";

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
