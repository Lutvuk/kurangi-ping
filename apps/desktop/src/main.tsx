import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import App from "./App";
import "./components/modules/modules.css";
import "./components/primitives/primitives.css";
import "./styles/index.css";

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
