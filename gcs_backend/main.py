import asyncio
from fastapi import FastAPI, WebSocket
from pydantic import BaseModel

app = FastAPI()


class Waypoint(BaseModel):
    lat: float
    lon: float
    alt: float


@app.post("/mission/start")
async def start_mission(wp: Waypoint):
    print(f"Mission started for waypoint: {wp.lat}, {wp.lon}, {wp.alt}")
    return {"status": "started", "waypoint": wp.model_dump()}


@app.websocket("/ws/telemetry")
async def websocket_telemetry(websocket: WebSocket):
    await websocket.accept()
    try:
        # Mocking telemetry stream coming from ZMQ
        while True:
            # Send protobuf-equivalent JSON
            await websocket.send_json({
                "lat": 37.7749,
                "lon": -122.4194,
                "alt": 50.0,
                "heading": 45.0,
                "tier": 1
            })
            await asyncio.sleep(0.1)
    except Exception as e:
        print(f"WebSocket closed: {e}")
