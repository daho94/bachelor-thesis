import "maptalks/dist/maptalks.css";
import * as maptalks from "maptalks";
import { MapOptions } from "maptalks";
import { useEffect, useRef, useState } from "react";

const baseOptions: MapOptions = {
  attribution: false,
  zoom: 15,
  minZoom: 3,
  maxZoom: 26,
  center: [11.772802012995772, 48.10616515447348],
  zoomControl: true,
};

export function MapTalks(props: { edges: any[][] }) {
  const [map, setMap] = useState<maptalks.Map>();
  const [layer, setLayer] = useState<maptalks.VectorLayer>(
    new maptalks.VectorLayer("vector")
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
    layer.addTo(map);
    mapDidRender.current = true;
    setMap(map);
    console.log("Map rendered");
  }, [baseOptions]);

  useEffect(() => {
    if (!mapDidRender.current) return;
    if (props.edges.length === 0) return;
    let polylines = props.edges.map((edge) => {
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
    layer.clear();
    layer.addGeometry(polylines);
  }, [props.edges]);

  return <div id="maptalks-id" className="maptalks-container"></div>;
}
