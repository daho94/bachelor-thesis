import { MapContainer, TileLayer } from "react-leaflet";
import "leaflet/dist/leaflet.css";
import {
  Circle,
  CircleMarker,
  Polygon,
  Polyline,
  Popup,
  Rectangle,
} from "react-leaflet";
import { LatLngBoundsExpression, LatLngExpression } from "leaflet";
import { useEffect, useState } from "react";

import { listen } from "@tauri-apps/api/event";

const center: LatLngExpression = [48.10616515447348, 11.772802012995772];

const polyline: LatLngExpression[] = [
  [51.505, -0.09],
  [51.51, -0.1],
  [51.51, -0.12],
];

const multiPolyline: LatLngExpression[][] = [
  [
    [51.5, -0.1],
    [51.5, -0.12],
    [51.52, -0.12],
  ],
  [
    [51.5, -0.05],
    [51.5, -0.06],
    [51.52, -0.06],
  ],
];

const fillBlueOptions = { fillColor: "blue" };
const blackOptions = { color: "black" };
const limeOptions = { color: "lime" };
const purpleOptions = { color: "purple" };
const redOptions = { color: "red" };

export function Map(props: { edges: LatLngExpression[][] }) {
  return (
    <MapContainer center={center} zoom={13} scrollWheelZoom={false}>
      <TileLayer
        attribution='&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors'
        url="https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png"
      />
      {props.edges.map((edge, idx) => (
        <Polyline key={idx} pathOptions={redOptions} positions={edge} />
      ))}
    </MapContainer>
  );
}
