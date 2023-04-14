import "maptalks/dist/maptalks.css";
import * as maptalks from "maptalks";
import { MapOptions } from "maptalks";
import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/tauri";

const baseOptions: MapOptions = {
  attribution: false,
  zoom: 15,
  minZoom: 3,
  maxZoom: 26,
  center: [11.772802012995772, 48.10616515447348],
  zoomControl: true,
};

export function MapTalks(props: { edges: any[][]; nodes: any[] }) {
  const [map, setMap] = useState<maptalks.Map>();
  const [layer, setLayer] = useState<maptalks.VectorLayer>(
    new maptalks.VectorLayer("vector")
  );
  const [pathLayer, setPathLayer] = useState<maptalks.VectorLayer>(
    new maptalks.VectorLayer("shortest-path")
  );

  const mapDidRender = useRef(false);

  // init map, which will not update until it's destroyed
  useEffect(() => {
    if (mapDidRender.current) return;

    let map = new maptalks.Map("maptalks-id", {
      ...baseOptions,
      baseLayer: new maptalks.TileLayer("base", {
        urlTemplate:
          "https://{s}.basemaps.cartocdn.com/light_all/{z}/{x}/{y}.png",
        subdomains: ["a", "b", "c", "d"],
        attribution:
          '&copy; <a href="http://osm.org">OpenStreetMap</a> contributors, &copy; <a href="https://carto.com/">CARTO</a>',
      }),
    });

    // Setup listener
    map.on("click", async (param) => {
      let { x: lng, y: lat } = param.coordinate;
      let latLng = [lat, lng];
      console.log(lng, lat);

      let { path, weight, duration, nodesSettled }: any = await invoke(
        "calc_path",
        {
          srcCoords: latLng,
          dstCoords: [48.10471649826582, 11.765805082376565],
        }
      );
      pathLayer.clear();

      if (path.length < 2) {
        return;
      }

      let lines = [];

      for (let i = 0; i < path.length - 1; i++) {
        lines.push([
          [path[i][0], path[i][1]],
          [path[i + 1][0], path[i + 1][1]],
        ]);
      }

      let multiline = new maptalks.MultiLineString(lines, {
        symbol: {
          lineColor: "#cf372b",
          lineWidth: 3,
        },
      }).setInfoWindow({
        content: `
        <div style="color:#f00">
          Calculation time: ${(duration * 1000).toFixed(2)} ms <br>
          Nodes settled: ${nodesSettled} <br>
          Weight: ${weight.toFixed(2)} s
        </div>`,
      });

      pathLayer.addGeometry(multiline);
      multiline.openInfoWindow();
    });

    // Add marker
    let point = new maptalks.Marker([11.765805082376565, 48.10471649826582], {
      visible: true,
      editable: true,
      cursor: "pointer",
      draggable: false,
      dragShadow: false, // display a shadow during dragging
      drawOnAxis: null, // force dragging stick on a axis, can be: x, y
    });

    layer.addGeometry(point);

    layer.addTo(map);
    pathLayer.addTo(map);
    mapDidRender.current = true;
    setMap(map);
    console.log("Map rendered");
  }, [baseOptions]);

  useEffect(() => {
    if (!mapDidRender.current) return;
    if (props.edges.length === 0) return;

    // Skip every second edge because they are duplicates (bidirectional)

    let polylines = props.edges
      .filter((_, i) => i % 2 === 0)
      .map((edge) => {
        return new maptalks.LineString(
          [
            [edge[0][1], edge[0][0]],
            [edge[1][1], edge[1][0]],
          ],
          {
            symbol: {
              lineColor: "#34495e",
              lineWidth: 1,
            },
          }
        );
      });
    // Remove all geometries from layer
    layer.addGeometry(polylines);
  }, [props.edges]);

  useEffect(() => {
    if (!mapDidRender.current) return;
    if (props.nodes.length === 0) return;

    let points = props.nodes.map((node) => {
      return new maptalks.Marker([node[1], node[2]], {
        visible: true,
        symbol: {
          markerType: "ellipse",
          markerFill: "#1bbc9b",
          markerWidth: 5,
          markerHeight: 5,
          markerText: parseInt(node[0]),
        },
      }).setInfoWindow({
        content: `
        <div style="color:#f00">
          NodeId: ${node[0]}<br>
          Lat: ${node[2].toFixed(3)}<br>
          Lon: ${node[1].toFixed(3)}<br>
        
        </div>`,
      });
    });

    layer.addGeometry(points);
  }, [props.nodes]);

  return <div id="maptalks-id" className="maptalks-container"></div>;
}
