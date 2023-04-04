import { useState } from "react";
import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/tauri";
import { open } from "@tauri-apps/api/dialog";
import "./App.css";
import { Map } from "./components/Map";
import { MapTalks } from "./components/MapTalks";
import { LatLngExpression } from "leaflet";

function App() {
  const [greetMsg, setGreetMsg] = useState("");
  const [path, setPath] = useState("");
  const [edges, setEdges] = useState<LatLngExpression[][]>([]);

  async function buildGraph() {
    setGreetMsg("Building graph...");
    const edges: LatLngExpression[][] = await invoke("create_graph_from_pbf", {
      path,
    });
    setGreetMsg("Done.");
    setEdges(edges);
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
      {/* <h1>Welcome to Tauri!</h1>

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

      <p>Click on the Tauri, Vite, and React logos to learn more.</p> */}

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
      {/* <div className="row"> */}
      {/* <Map edges={edges}></Map> */}
      <MapTalks edges={edges}></MapTalks>
      {/* </div> */}
    </div>
  );
}

export default App;
