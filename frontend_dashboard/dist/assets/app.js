// 3D Visualizer for Landing Section
function initLandingVisualizer() {
    const container = document.getElementById('landing-canvas-container');
    if (!container) return;

    const scene = new THREE.Scene();
    scene.background = new THREE.Color(0xeef2f7);

    const camera = new THREE.PerspectiveCamera(75, container.clientWidth / container.clientHeight, 0.1, 1000);
    const renderer = new THREE.WebGLRenderer({ antialias: true });
    
    renderer.setSize(container.clientWidth, container.clientHeight);
    container.appendChild(renderer.domElement);

    // Torus knot for AI containment visual (similar to reference)
    const geometry = new THREE.TorusKnotGeometry(10, 3, 100, 16);
    const material = new THREE.MeshBasicMaterial({ 
        color: 0x8b5cf6, // Purple-ish accent
        wireframe: true,
        transparent: true,
        opacity: 0.5
    });
    const torusKnot = new THREE.Mesh(geometry, material);
    scene.add(torusKnot);

    camera.position.z = 30;

    function animate() {
        requestAnimationFrame(animate);
        torusKnot.rotation.x += 0.005;
        torusKnot.rotation.y += 0.01;
        renderer.render(scene, camera);
    }

    animate();

    window.addEventListener('resize', () => {
        camera.aspect = container.clientWidth / container.clientHeight;
        camera.updateProjectionMatrix();
        renderer.setSize(container.clientWidth, container.clientHeight);
    });
}

// GCS Dashboard Logic
function initGCS() {
    const canvas = document.getElementById('map-canvas');
    if (!canvas) return;
    
    const ctx = canvas.getContext('2d');
    let latestTelemetry = null;

    // Resize canvas to match container
    function resizeCanvas() {
        const rect = canvas.parentElement.getBoundingClientRect();
        canvas.width = rect.width;
        canvas.height = rect.height;
    }
    window.addEventListener('resize', resizeCanvas);
    resizeCanvas();

    // Map Drawing Logic
    function drawMap() {
        ctx.clearRect(0, 0, canvas.width, canvas.height);
        
        // Draw grid
        ctx.strokeStyle = '#1e293b';
        ctx.lineWidth = 1;
        for(let i = 0; i < canvas.width; i+=50) {
            ctx.beginPath(); ctx.moveTo(i, 0); ctx.lineTo(i, canvas.height); ctx.stroke();
        }
        for(let i = 0; i < canvas.height; i+=50) {
            ctx.beginPath(); ctx.moveTo(0, i); ctx.lineTo(canvas.width, i); ctx.stroke();
        }

        if (!latestTelemetry) return;

        // Coordinate transformation: center map at (0,0) mapped to center of canvas
        const cx = canvas.width / 2;
        const cy = canvas.height / 2;
        const scale = 0.2; // 1 unit = 0.2 pixels

        function toScreen(x, y) {
            return [cx + x * scale, cy - y * scale];
        }

        // Draw Anchors
        ctx.fillStyle = '#38bdf8';
        latestTelemetry.anchors.forEach(a => {
            const [sx, sy] = toScreen(a.x, a.y);
            ctx.beginPath();
            ctx.arc(sx, sy, 5, 0, 2*Math.PI);
            ctx.fill();
        });

        // Draw Swarm (UAVs)
        ctx.fillStyle = '#22c55e';
        latestTelemetry.swarm.forEach((uav, i) => {
            const [sx, sy] = toScreen(uav.x, uav.y);
            ctx.beginPath();
            ctx.arc(sx, sy, 8, 0, 2*Math.PI);
            ctx.fill();
            
            // Draw EKF Estimate
            ctx.strokeStyle = '#facc15'; // Yellow
            ctx.beginPath();
            const estX = latestTelemetry.ekf_estimates[i]?.x || uav.x;
            const estY = latestTelemetry.ekf_estimates[i]?.y || uav.y;
            const [ex, ey] = toScreen(estX, estY);
            ctx.arc(ex, ey, 10, 0, 2*Math.PI);
            ctx.stroke();
        });

        // Draw Jammer
        if (latestTelemetry.jammer.active) {
            ctx.fillStyle = 'rgba(239, 68, 68, 0.2)'; // Red Area
            const [jx, jy] = toScreen(latestTelemetry.jammer.x, latestTelemetry.jammer.y);
            ctx.beginPath();
            ctx.arc(jx, jy, 150 * scale, 0, 2*Math.PI); // Range approx
            ctx.fill();
            
            ctx.fillStyle = '#ef4444';
            ctx.beginPath();
            ctx.arc(jx, jy, 6, 0, 2*Math.PI);
            ctx.fill();
        }

        // Draw Hunter
        if (latestTelemetry.hunter.active) {
            ctx.fillStyle = '#f97316'; // Orange
            const [hx, hy] = toScreen(latestTelemetry.hunter.x, latestTelemetry.hunter.y);
            ctx.beginPath();
            ctx.moveTo(hx, hy-8);
            ctx.lineTo(hx-6, hy+6);
            ctx.lineTo(hx+6, hy+6);
            ctx.fill();
        }
    }

    // Animation Loop
    function renderLoop() {
        drawMap();
        requestAnimationFrame(renderLoop);
    }
    renderLoop();

    // WebSocket Setup
    const telemetryOut = document.getElementById('telemetry-out');
    const wsUrl = `ws://${window.location.host}/ws/telemetry`;
    let ws = new WebSocket(wsUrl);

    ws.onmessage = (event) => {
        const data = JSON.parse(event.data);
        latestTelemetry = data;
        
        // Update telemetry panel text (prettified)
        telemetryOut.textContent = JSON.stringify(data, null, 2);
    };

    ws.onclose = () => {
        telemetryOut.textContent = "Disconnected from C2 Server.";
    };

    // UI Controls
    document.getElementById('btn-jammer-toggle').addEventListener('click', async (e) => {
        const isActive = e.target.classList.toggle('active');
        if(isActive) e.target.textContent = 'Disable Jammer';
        else e.target.textContent = 'Enable Jammer';
        
        await fetch('/api/control/jammer', {
            method: 'POST',
            headers: {'Content-Type': 'application/json'},
            body: JSON.stringify({ active: isActive, power_dbm: 45, x: 200, y: 300 })
        });
    });

    document.getElementById('btn-hunter-toggle').addEventListener('click', async (e) => {
        const isActive = e.target.classList.toggle('active');
        if(isActive) e.target.textContent = 'Disable Hunter';
        else e.target.textContent = 'Enable Hunter';
        
        await fetch('/api/control/hunter', {
            method: 'POST',
            headers: {'Content-Type': 'application/json'},
            body: JSON.stringify({ active: isActive, x: -100, y: -200 })
        });
    });

    const windSlider = document.getElementById('wind-slider');
    const windVal = document.getElementById('wind-val');
    windSlider.addEventListener('input', async (e) => {
        const level = parseInt(e.target.value);
        windVal.textContent = level;
        await fetch('/api/control/wind', {
            method: 'POST',
            headers: {'Content-Type': 'application/json'},
            body: JSON.stringify({ level: level })
        });
    });

    document.getElementById('btn-reset').addEventListener('click', async () => {
        await fetch('/api/control/reset', { method: 'POST' });
    });
}

// Initialize on load
document.addEventListener('DOMContentLoaded', () => {
    initLandingVisualizer();
    initGCS();
});
