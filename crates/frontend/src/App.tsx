import { useState } from "react";
import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/tauri";
import { open } from "@tauri-apps/api/dialog";
import "./App.css";

function App() {
  const [greetMsg, setGreetMsg] = useState("");
  const [path, setPath] = useState("");

  async function buildGraph() {
    setGreetMsg("Building graph...");
    setGreetMsg(await invoke("create_graph_from_pbf", { path }));
  }

  async function selectFile() {
    const selected = await open({
      filters: [{ name: "PBF", extensions: ["pbf"] }],
    });
    if (Array.isArray(selected)) {
      // user selected multiple directories
    } else if (selected === null) {
      // user cancelled the selection
    } else {
      // user selected a single directory
      console.log(selected);
      setPath(selected);
    }
  }

  return (
    <div className="container">
      <h1>Welcome to Tauri!</h1>

      <div className="row">
        <a href="https://vitejs.dev" target="_blank">
          <img src="/vite.svg" className="logo vite" alt="Vite logo" />
        </a>
        <a href="https://tauri.app" target="_blank">
          <img src="/tauri.svg" className="logo tauri" alt="Tauri logo" />
        </a>
        <a href="https://reactjs.org" target="_blank">
          <img src={reactLogo} className="logo react" alt="React logo" />
        </a>
      </div>

      <p>Click on the Tauri, Vite, and React logos to learn more.</p>

      <div className="row">
        <div>
          <input
            id="path-input"
            placeholder="Please select *.pbf file"
            value={path}
            onClick={(e) => selectFile()}
          />
          <button disabled={!path} type="button" onClick={() => buildGraph()}>
            Build Graph
          </button>
        </div>
      </div>
      <p>{greetMsg}</p>
    </div>
  );
}

export default App;
