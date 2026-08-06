import * as maplibregl from 'maplibre-gl';
import 'maplibre-gl/dist/maplibre-gl.css';

const map = new maplibregl.Map({
    container: 'map',
    style: 'https://basemaps.cartocdn.com/gl/dark-matter-gl-style/style.json',
    center: [-122.4194, 37.7749],
    zoom: 14
});

const ws = new WebSocket("ws://localhost:8000/ws/telemetry");

let droneMarker: maplibregl.Marker | null = null;

ws.onmessage = (event) => {
    const data = JSON.parse(event.data);
    const lngLat: [number, number] = [data.lon, data.lat];
    
    if (!droneMarker) {
        droneMarker = new maplibregl.Marker({ color: "#FF0000" })
            .setLngLat(lngLat)
            .addTo(map);
    } else {
        droneMarker.setLngLat(lngLat);
    }
};

// Heatmap / Neural Activations placeholder overlay
map.on('load', () => {
    map.addSource('heatmap-source', {
        type: 'geojson',
        data: {
            type: 'FeatureCollection',
            features: []
        }
    });
    map.addLayer({
        id: 'heatmap-layer',
        type: 'heatmap',
        source: 'heatmap-source',
        paint: {
            'heatmap-weight': 1,
            'heatmap-intensity': 1,
            'heatmap-color': [
                'interpolate',
                ['linear'],
                ['heatmap-density'],
                0, 'rgba(33,102,172,0)',
                0.2, 'rgb(103,169,207)',
                0.4, 'rgb(209,229,240)',
                0.6, 'rgb(253,219,199)',
                0.8, 'rgb(239,138,98)',
                1, 'rgb(178,24,43)'
            ],
            'heatmap-radius': 30,
            'heatmap-opacity': 0.8
        }
    });
});
