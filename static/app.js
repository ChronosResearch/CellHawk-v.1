/**
 * CELLHAWK GCS Digital Twin UI Engine.
 * Handles WebSocket telemetry streaming, Canvas 2D map rendering,
 * EKF tier HUD status badges, 400 Hz PID motor bars, and C2 control dispatch.
 */

document.addEventListener('DOMContentLoaded', () => {
    // DOM Elements
    const canvas = document.getElementById('mapCanvas');
    const ctx = canvas.getContext('2d');
    const chartCanvas = document.getElementById('chartRmsCanvas');
    const chartCtx = chartCanvas.getContext('2d');

    // HUD Elements
    const tierStatusPill = document.getElementById('tierStatusPill');
    const hudJnr = document.getElementById('hudJnr');
    const hudRms = document.getElementById('hudRms');
    const hudAlpha = document.getElementById('hudAlpha');
    const hudSimTime = document.getElementById('hudSimTime');

    // Control Elements
    const btnToggleJammer = document.getElementById('btnToggleJammer');
    const sliderJammerPower = document.getElementById('sliderJammerPower');
    const valJammerPower = document.getElementById('valJammerPower');

    const btnToggleHunter = document.getElementById('btnToggleHunter');
    const btnResetSim = document.getElementById('btnResetSim');

    const chkShowTowers = document.getElementById('chkShowTowers');
    const chkShowJammer = document.getElementById('chkShowJammer');
    const chkShowHeatmap = document.getElementById('chkShowHeatmap');
    const chkShowLidar = document.getElementById('chkShowLidar');
    const chkShowDanger = document.getElementById('chkShowDanger');

    // Telemetry Elements
    const valIntentHeading = document.getElementById('valIntentHeading');
    const valClimbRate = document.getElementById('valClimbRate');
    const valBattery = document.getElementById('valBattery');

    const pwmFL = document.getElementById('pwmFL');
    const pwmFR = document.getElementById('pwmFR');
    const pwmRL = document.getElementById('pwmRL');
    const pwmRR = document.getElementById('pwmRR');

    const barFL = document.getElementById('barFL');
    const barFR = document.getElementById('barFR');
    const barRL = document.getElementById('barRL');
    const barRR = document.getElementById('barRR');

    // Application State
    let telemetryData = null;
    let flightHistory = [];
    let rmsHistory = [];
    let ws = null;

    // Canvas scaling & transformation params
    const mapScale = 0.45; // pixels per meter
    const mapOriginX = 450;
    const mapOriginY = 350;

    function resizeCanvas() {
        canvas.width = canvas.parentElement.clientWidth;
        canvas.height = canvas.parentElement.clientHeight;
        chartCanvas.width = chartCanvas.parentElement.clientWidth - 28;
    }
    window.addEventListener('resize', resizeCanvas);
    resizeCanvas();

    // Map coordinate transformations (ENU -> Canvas)
    function worldToCanvas(x, y) {
        return {
            cx: mapOriginX + x * mapScale,
            cy: mapOriginY - y * mapScale // Canvas Y inverted
        };
    }

    // Connect WebSocket Telemetry Stream
    function initWebSocket() {
        const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        const wsUrl = `${protocol}//${window.location.host}/ws/telemetry`;

        ws = new WebSocket(wsUrl);

        ws.onopen = () => {
            console.log('WebSocket C2 Link Connected');
        };

        ws.onmessage = (event) => {
            telemetryData = JSON.parse(event.data);
            updateUI(telemetryData);
            renderMap(telemetryData);
            renderRmsChart();
        };

        ws.onclose = () => {
            console.warn('WebSocket C2 Link Closed. Retrying in 2s...');
            setTimeout(initWebSocket, 2000);
        };

        ws.onerror = (err) => {
            console.error('WebSocket Error:', err);
        };
    }

    // Update Telemetry & HUD Widgets
    function updateUI(data) {
        if (!data || !data.nodes || data.nodes.length === 0) return;

        const node = data.nodes[0]; // Primary UAV node

        hudSimTime.innerHTML = `${data.simulation_time.toFixed(1)} <small>s</small>`;
        hudJnr.innerHTML = `${node.jnr_db.toFixed(1)} <small>dB</small>`;
        hudRms.innerHTML = `${node.rms_error_m.toFixed(1)} <small>m</small>`;
        hudAlpha.innerText = node.handover_alpha.toFixed(2);

        // Update Tier Status Badge
        tierStatusPill.className = 'status-pill';
        if (node.ekf_tier === 1) {
            tierStatusPill.classList.add('tier-1');
            tierStatusPill.innerHTML = '<span class="pill-dot glow-green"></span><span class="pill-label">TIER 1: GNSS ACTIVE</span>';
        } else if (node.ekf_tier === 2) {
            tierStatusPill.classList.add('tier-2');
            tierStatusPill.innerHTML = '<span class="pill-dot glow-amber"></span><span class="pill-label">TIER 2: CELLULAR RSSI</span>';
        } else {
            tierStatusPill.classList.add('tier-3');
            tierStatusPill.innerHTML = '<span class="pill-dot glow-purple"></span><span class="pill-label">TIER 3: VISUAL SLAM</span>';
        }

        // Flight history trail
        flightHistory.push({ x: node.position_true[0], y: node.position_true[1] });
        if (flightHistory.length > 300) flightHistory.shift();

        // RMS history for live chart
        rmsHistory.push(node.rms_error_m);
        if (rmsHistory.length > 50) rmsHistory.shift();

        // Telemetry cards
        valIntentHeading.innerText = `${node.attitude_heading_deg.toFixed(1)}°`;
        valClimbRate.innerText = `${node.velocity_true[2].toFixed(1)} m/s`;
        valBattery.innerText = `${node.battery_v.toFixed(2)} V`;

        // Motor PWMs
        if (node.motor_pwms) {
            pwmFL.innerText = `${Math.round(node.motor_pwms.m1)} us`;
            pwmFR.innerText = `${Math.round(node.motor_pwms.m2)} us`;
            pwmRL.innerText = `${Math.round(node.motor_pwms.m4)} us`;
            pwmRR.innerText = `${Math.round(node.motor_pwms.m3)} us`;

            barFL.style.width = `${((node.motor_pwms.m1 - 1000) / 1000) * 100}%`;
            barFR.style.width = `${((node.motor_pwms.m2 - 1000) / 1000) * 100}%`;
            barRL.style.width = `${((node.motor_pwms.m4 - 1000) / 1000) * 100}%`;
            barRR.style.width = `${((node.motor_pwms.m3 - 1000) / 1000) * 100}%`;
        }

        // Jammer & Hunter button sync
        btnToggleJammer.innerText = data.jammer_active ? 'JAMMER ON' : 'JAMMER OFF';
        btnToggleJammer.classList.toggle('active', data.jammer_active);

        btnToggleHunter.innerText = data.hunter_active ? 'DESPAWN HUNTER' : 'SPAWN HUNTER';
        btnToggleHunter.classList.toggle('active', data.hunter_active);
    }

    // Canvas 2D Renderer
    function renderMap(data) {
        ctx.clearRect(0, 0, canvas.width, canvas.height);

        // 1. Draw Grid Lines & Tactical Axis
        ctx.strokeStyle = 'rgba(255, 255, 255, 0.04)';
        ctx.lineWidth = 1;
        const gridSize = 50 * mapScale;
        for (let x = 0; x < canvas.width; x += gridSize) {
            ctx.beginPath();
            ctx.moveTo(x, 0);
            ctx.lineTo(x, canvas.height);
            ctx.stroke();
        }
        for (let y = 0; y < canvas.height; y += gridSize) {
            ctx.beginPath();
            ctx.moveTo(0, y);
            ctx.lineTo(canvas.width, y);
            ctx.stroke();
        }

        if (!data) return;

        const node = data.nodes[0];

        // 2. Render CORTEX Neural Risk Heatmap Overlay
        if (chkShowHeatmap.checked && node && node.neural_risk_heatmap) {
            const grid = node.neural_risk_heatmap;
            const stepMeters = 20.0;
            const startX = node.position_true[0] - 160.0;
            const startY = node.position_true[1] - 160.0;

            for (let r = 0; r < 16; r++) {
                for (let c = 0; c < 16; c++) {
                    const val = grid[r][c];
                    if (val > 0.05) {
                        const pos = worldToCanvas(startX + c * stepMeters, startY + r * stepMeters);
                        ctx.fillStyle = `rgba(244, 63, 94, ${val * 0.18})`;
                        ctx.fillRect(pos.cx, pos.cy - stepMeters * mapScale, stepMeters * mapScale, stepMeters * mapScale);
                    }
                }
            }
        }

        // 3. Render 4G/LTE Cell Towers & RSSI Wave Heat Rings
        if (chkShowTowers.checked && data.towers) {
            data.towers.forEach(t => {
                const pos = worldToCanvas(t.x, t.y);

                // Range heat ring
                ctx.strokeStyle = 'rgba(0, 243, 255, 0.12)';
                ctx.lineWidth = 1;
                ctx.beginPath();
                ctx.arc(pos.cx, pos.cy, 180 * mapScale, 0, Math.PI * 2);
                ctx.stroke();

                // Vector line from drone to tower
                if (node) {
                    const dronePos = worldToCanvas(node.position_true[0], node.position_true[1]);
                    ctx.strokeStyle = 'rgba(0, 243, 255, 0.15)';
                    ctx.setLineDash([4, 4]);
                    ctx.beginPath();
                    ctx.moveTo(dronePos.cx, dronePos.cy);
                    ctx.lineTo(pos.cx, pos.cy);
                    ctx.stroke();
                    ctx.setLineDash([]);
                }

                // Tower icon
                ctx.fillStyle = '#00f3ff';
                ctx.beginPath();
                ctx.arc(pos.cx, pos.cy, 6, 0, Math.PI * 2);
                ctx.fill();

                ctx.fillStyle = '#94a3b8';
                ctx.font = '10px JetBrains Mono';
                ctx.fillText(t.id, pos.cx + 10, pos.cy + 3);
            });
        }

        // 4. Render EA Jammer Dome
        if (chkShowJammer.checked && data.jammer_active) {
            const jPos = worldToCanvas(data.jammer_pos[0], data.jammer_pos[1]);
            const jammerRadius = 500 * mapScale;

            const grad = ctx.createRadialGradient(jPos.cx, jPos.cy, 10, jPos.cx, jPos.cy, jammerRadius);
            grad.addColorStop(0, 'rgba(244, 63, 94, 0.4)');
            grad.addColorStop(0.7, 'rgba(244, 63, 94, 0.15)');
            grad.addColorStop(1, 'rgba(244, 63, 94, 0.0)');

            ctx.fillStyle = grad;
            ctx.beginPath();
            ctx.arc(jPos.cx, jPos.cy, jammerRadius, 0, Math.PI * 2);
            ctx.fill();

            ctx.strokeStyle = 'rgba(244, 63, 94, 0.6)';
            ctx.lineWidth = 2;
            ctx.beginPath();
            ctx.arc(jPos.cx, jPos.cy, jammerRadius, 0, Math.PI * 2);
            ctx.stroke();

            ctx.fillStyle = '#f43f5e';
            ctx.font = 'bold 11px JetBrains Mono';
            ctx.fillText('EA JAMMER DOME', jPos.cx - 45, jPos.cy - 10);
        }

        // 5. Render Danger Grid Stigmergic Pheromones
        if (chkShowDanger.checked && data.danger_grid) {
            data.danger_grid.forEach(p => {
                const pos = worldToCanvas(p.x, p.y);
                ctx.fillStyle = `rgba(245, 158, 11, ${p.threat * 0.4})`;
                ctx.beginPath();
                ctx.arc(pos.cx, pos.cy, 12, 0, Math.PI * 2);
                ctx.fill();
            });
        }

        // 6. Render Hunter Drone & TPN Pursuit Vector
        if (data.hunter_active && data.hunter_pos) {
            const hPos = worldToCanvas(data.hunter_pos[0], data.hunter_pos[1]);

            // Forward camera FOV wedge
            const hHeading = data.hunter_heading_deg * (Math.PI / 180);
            const fovAngle = 60 * (Math.PI / 180);

            ctx.fillStyle = 'rgba(244, 63, 94, 0.15)';
            ctx.beginPath();
            ctx.moveTo(hPos.cx, hPos.cy);
            ctx.arc(hPos.cx, hPos.cy, 120 * mapScale, hHeading - fovAngle / 2, hHeading + fovAngle / 2);
            ctx.closePath();
            ctx.fill();

            // Hunter icon
            ctx.fillStyle = '#f43f5e';
            ctx.beginPath();
            ctx.arc(hPos.cx, hPos.cy, 7, 0, Math.PI * 2);
            ctx.fill();

            ctx.fillStyle = '#ffffff';
            ctx.font = 'bold 10px JetBrains Mono';
            ctx.fillText('TPN HUNTER', hPos.cx + 12, hPos.cy + 4);
        }

        // 7. Render Primary UAV Node & Flight Trail
        if (node) {
            // Draw Flight Trail
            if (flightHistory.length > 1) {
                ctx.strokeStyle = 'rgba(16, 185, 129, 0.5)';
                ctx.lineWidth = 2;
                ctx.beginPath();
                for (let i = 0; i < flightHistory.length; i++) {
                    const p = worldToCanvas(flightHistory[i].x, flightHistory[i].y);
                    if (i === 0) ctx.moveTo(p.cx, p.cy);
                    else ctx.lineTo(p.cx, p.cy);
                }
                ctx.stroke();
            }

            const dPos = worldToCanvas(node.position_true[0], node.position_true[1]);
            const ekfPos = worldToCanvas(node.ekf_position[0], node.ekf_position[1]);

            // EKF Estimated Position Marker (Orange ring)
            ctx.strokeStyle = '#f59e0b';
            ctx.lineWidth = 1.5;
            ctx.setLineDash([3, 3]);
            ctx.beginPath();
            ctx.arc(ekfPos.cx, ekfPos.cy, node.rms_error_m * mapScale, 0, Math.PI * 2);
            ctx.stroke();
            ctx.setLineDash([]);

            // 360° 8-sector LiDAR Rays
            if (chkShowLidar.checked) {
                const headingRad = node.attitude_heading_deg * (Math.PI / 180);
                for (let i = 0; i < 8; i++) {
                    const rayAngle = headingRad + i * (Math.PI / 4);
                    const rayLen = 40.0 * mapScale;
                    ctx.strokeStyle = 'rgba(16, 185, 129, 0.3)';
                    ctx.beginPath();
                    ctx.moveTo(dPos.cx, dPos.cy);
                    ctx.lineTo(dPos.cx + Math.cos(rayAngle) * rayLen, dPos.cy - Math.sin(rayAngle) * rayLen);
                    ctx.stroke();
                }
            }

            // UAV Icon & Heading Vector
            ctx.fillStyle = '#10b981';
            ctx.beginPath();
            ctx.arc(dPos.cx, dPos.cy, 9, 0, Math.PI * 2);
            ctx.fill();

            const headingRad = node.attitude_heading_deg * (Math.PI / 180);
            ctx.strokeStyle = '#ffffff';
            ctx.lineWidth = 2.5;
            ctx.beginPath();
            ctx.moveTo(dPos.cx, dPos.cy);
            ctx.lineTo(dPos.cx + Math.cos(headingRad) * 22, dPos.cy - Math.sin(headingRad) * 22);
            ctx.stroke();

            ctx.fillStyle = '#ffffff';
            ctx.font = 'bold 11px Inter';
            ctx.fillText(node.drone_id, dPos.cx + 14, dPos.cy - 10);
        }
    }

    // Render Live RMS Error Chart Canvas
    function renderRmsChart() {
        chartCtx.clearRect(0, 0, chartCanvas.width, chartCanvas.height);

        if (rmsHistory.length < 2) return;

        const w = chartCanvas.width;
        const h = chartCanvas.height;

        chartCtx.strokeStyle = 'rgba(255, 255, 255, 0.1)';
        chartCtx.lineWidth = 1;
        chartCtx.beginPath();
        chartCtx.moveTo(0, h - 20);
        chartCtx.lineTo(w, h - 20);
        chartCtx.stroke();

        const stepX = w / (rmsHistory.length - 1);
        const maxVal = 60.0; // 60 meters max chart scale

        chartCtx.strokeStyle = '#00f3ff';
        chartCtx.lineWidth = 2;
        chartCtx.beginPath();

        for (let i = 0; i < rmsHistory.length; i++) {
            const x = i * stepX;
            const y = h - 20 - (Math.min(rmsHistory[i], maxVal) / maxVal) * (h - 30);
            if (i === 0) chartCtx.moveTo(x, y);
            else chartCtx.lineTo(x, y);
        }
        chartCtx.stroke();
    }

    // Event Handlers for Controls
    btnToggleJammer.addEventListener('click', async () => {
        const isCurrentlyActive = btnToggleJammer.classList.contains('active');
        await fetch('/api/control/jammer', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                active: !isCurrentlyActive,
                power_dbm: parseFloat(sliderJammerPower.value),
                x: 400.0,
                y: 500.0,
            }),
        });
    });

    sliderJammerPower.addEventListener('input', () => {
        valJammerPower.innerText = `${sliderJammerPower.value} dBm`;
    });

    btnToggleHunter.addEventListener('click', async () => {
        const isCurrentlyActive = btnToggleHunter.classList.contains('active');
        await fetch('/api/control/hunter', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                active: !isCurrentlyActive,
                x: -300.0,
                y: -200.0,
            }),
        });
    });

    document.querySelectorAll('.btn-chip[data-wind]').forEach(chip => {
        chip.addEventListener('click', async () => {
            document.querySelectorAll('.btn-chip[data-wind]').forEach(c => c.classList.remove('active'));
            chip.classList.add('active');
            const windLvl = parseInt(chip.dataset.wind);
            await fetch('/api/control/wind', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ level: windLvl }),
            });
        });
    });

    document.querySelectorAll('.btn-chip[data-swarm]').forEach(chip => {
        chip.addEventListener('click', async () => {
            document.querySelectorAll('.btn-chip[data-swarm]').forEach(c => c.classList.remove('active'));
            chip.classList.add('active');
            const swarmSize = parseInt(chip.dataset.swarm);
            await fetch('/api/control/swarm', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ swarm_size: swarmSize }),
            });
        });
    });

    btnResetSim.addEventListener('click', async () => {
        flightHistory = [];
        rmsHistory = [];
        await fetch('/api/control/reset', { method: 'POST' });
    });

    // Start WebSocket
    initWebSocket();
});
