use crate::methodology::configuration::CONFIG;
use crate::methodology::globals::get_execution_root_str;
use crate::methodology::goal_target_result::GoalTargetResult;
use std::fs;
use std::path::Path;

/// Generates the goals simulator viewer HTML file with embedded goals data
pub fn generate_goals_viewer(goals_targets_details: &[Vec<GoalTargetResult>]) {
    let output_dir = get_execution_root_str();
    let filename = "goals_simulator_v4_viewer.html";
    let filepath = Path::new(&output_dir).join("Goals/").join(filename);

    // Get the simulation configs
    let simulation_configs = CONFIG.goals.clone();

    // Convert to JSON
    let configs_json = format!(
        "{{\"simulation_configs\": {}}}",
        serde_json::to_string_pretty(&simulation_configs)
            .expect("Failed to serialize simulation configs to JSON")
    );
    // Convert to JSON
    let result_json = serde_json::to_string_pretty(&goals_targets_details)
        .expect("Failed to serialize goals targets details to JSON"); // <-- Corrected expect message

    // Generate HTML content with embedded data
    let html_content = generate_goals_viewer_html(&configs_json, &result_json);

    // Write to file
    fs::write(&filepath, html_content).unwrap_or_else(|e| {
        panic!("Failed to write goals viewer to {:?}: {}", filepath, e);
    });

    println!("✅ Generated goals simulator viewer: {}", filename);
}

fn generate_goals_viewer_html(configs_json: &str, result_json: &str) -> String {
    // Wrap the entire template in the format! macro
    // Inject configs_json and result_json directly as JS objects/arrays
    format!(
        r##"<!DOCTYPE html>
<html lang="en">
    <head>
        <meta charset="UTF-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1.0" />
        <title>Multi-Robot Sim Editor (Multi-Grid)</title>
        <link
            href="https://cdn.jsdelivr.net/npm/bootstrap@5.3.0/dist/css/bootstrap.min.css"
            rel="stylesheet"
        />
        <style>
            /* --- Basic Layout & Body --- */
            body {{
                margin: 0;
                padding: 10px;
                font-family: system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI",
                    Roboto, Oxygen, Ubuntu, Cantarell, "Open Sans", "Helvetica Neue",
                    sans-serif;
                height: 100vh;
                display: flex;
                flex-direction: column;
                background-color: #f4f7f9;
            }}
            h2 {{
                text-align: center;
                margin-bottom: 20px;
                color: #333;
                font-weight: 600;
            }}
            .main-container {{
                display: flex;
                flex-grow: 1;
                gap: 0; /* Remove gap to make resize handle seamless */
                max-height: calc(100vh - 85px); /* Adjusted for title */
                overflow: hidden; /* Prevent body scroll */
                position: relative;
            }}

            /* --- Resize Handle --- */
            .resize-handle {{
                width: 8px;
                background: linear-gradient(to right, #e9ecef, #dee2e6, #e9ecef);
                cursor: col-resize;
                position: relative;
                display: flex;
                align-items: center;
                justify-content: center;
                border-left: 1px solid #dee2e6;
                border-right: 1px solid #dee2e6;
                transition: background 0.2s ease;
                user-select: none;
            }}

            .resize-handle:hover {{
                background: linear-gradient(to right, #dee2e6, #ced4da, #dee2e6);
            }}

            .resize-handle:active {{
                background: linear-gradient(to right, #ced4da, #adb5bd, #ced4da);
            }}

            /* --- Toggle Button --- */
            .toggle-editor-btn {{
                position: absolute;
                top: 50%;
                left: 50%;
                transform: translate(-50%, -50%);
                width: 28px;
                height: 44px;
                background: linear-gradient(145deg, #ffffff, #f0f2f5);
                border: 2px solid #dee2e6;
                border-radius: 14px;
                cursor: pointer;
                display: flex;
                align-items: center;
                justify-content: center;
                font-size: 13px;
                font-weight: 600;
                color: #6c757d;
                transition: all 0.3s cubic-bezier(0.25, 0.8, 0.25, 1);
                z-index: 10;
                box-shadow: 0 3px 8px rgba(0, 0, 0, 0.12), 0 1px 3px rgba(0, 0, 0, 0.08);
            }}

            .toggle-editor-btn:hover {{
                background: linear-gradient(145deg, #f8f9fa, #e9ecef);
                color: #495057;
                transform: translate(-50%, -50%) scale(1.08);
                box-shadow: 0 5px 15px rgba(0, 0, 0, 0.2), 0 3px 6px rgba(0, 0, 0, 0.15);
                border-color: #adb5bd;
            }}

            .toggle-editor-btn:active {{
                transform: translate(-50%, -50%) scale(0.95);
                box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
            }}

            /* --- Left Column (Simulator Frames) --- */
            #simulator-column {{
                flex: 1;
                display: flex;
                flex-direction: column;
                border: 1px solid #d1d9e0;
                border-radius: 8px;
                background-color: #ffffff;
                overflow-y: auto; /* Make THIS scrollable */
                position: relative; /* Needed for drop indicator */
                padding-bottom: 15px;
                box-shadow: 0 2px 5px rgba(0, 0, 0, 0.05);
                scrollbar-width: thin;
                scrollbar-color: #adb5bd #f1f3f5;
            }}
            #simulator-column::-webkit-scrollbar {{
                width: 8px;
            }}
            #simulator-column::-webkit-scrollbar-track {{
                background: #f1f3f5;
                border-radius: 4px;
            }}
            #simulator-column::-webkit-scrollbar-thumb {{
                background-color: #adb5bd;
                border-radius: 4px;
                border: 2px solid #f1f3f5;
            }}
            #simulator-column::-webkit-scrollbar-thumb:hover {{
                background-color: #868e96;
            }}

            /* --- Individual Simulation Frame Styling --- */
            .simulation-frame {{
                border: 1px solid #d1d9e0;
                border-radius: 6px;
                padding: 10px;
                margin: 0 15px 12px 15px;
                background-color: #fcfdff;
                position: relative;
                display: flex;
                flex-direction: column; /* Changed to column */
                gap: 10px;
                flex-shrink: 0;
                transition: opacity 0.2s ease-in-out, box-shadow 0.2s ease-in-out;
                box-shadow: 0 2px 4px rgba(0, 0, 0, 0.06);
            }}
            .simulation-frame:hover {{
                box-shadow: 0 4px 8px rgba(0, 0, 0, 0.08);
                border-color: #a8b5c1;
            }}

            /* --- Frame Header --- */
            .frame-header {{
                width: 100%;
                display: flex;
                align-items: center;
                justify-content: space-between;
                margin-bottom: 5px; /* Reduced margin */
                padding-bottom: 5px;
                border-bottom: 1px solid #eee;
            }}

            /* --- Title (h4) --- */
            .simulation-frame h4 {{
                text-align: left;
                margin: 0;
                font-size: 1.05em;
                color: #444;
                font-weight: 600;
                white-space: nowrap;
                overflow: hidden;
                text-overflow: ellipsis;
                flex-grow: 1;
                padding: 0 5px;
            }}

            /* --- NEW: Grids Wrapper with Horizontal Scroll --- */
            .simulation-frame .grids-wrapper {{
                display: flex;
                overflow-x: auto; /* Enable horizontal scroll */
                width: 100%;
                gap: 15px;
                padding-bottom: 15px; /* Space for scrollbar */
                align-items: flex-start; /* Align tops of canvases */
                scrollbar-width: thin;
                scrollbar-color: #adb5bd #f1f3f5;
            }}
            .simulation-frame .grids-wrapper::-webkit-scrollbar {{
                height: 8px; /* Horizontal scrollbar height */
            }}
            .simulation-frame .grids-wrapper::-webkit-scrollbar-track {{
                background: #f1f3f5;
                border-radius: 4px;
            }}
            .simulation-frame .grids-wrapper::-webkit-scrollbar-thumb {{
                background-color: #adb5bd;
                border-radius: 4px;
                border: 2px solid #f1f3f5;
            }}

            /* --- Canvas/Boundary Styles --- */
            .simulation-frame .frame-canvas-container {{
                text-align: center;
                min-width: 305px;
                flex-shrink: 0; /* Prevent containers from shrinking */
                position: relative; /* For positioning */
                border-radius: 6px;
                transition: border-color 0.2s ease;
                display: flex;
                flex-direction: column;
                align-items: center;
                gap: 8px;
            }}
            .simulation-frame .frame-canvas-container.target-grid {{
                flex-direction: row;
                align-items: center;
                gap: 16px;
                text-align: left;
            }}
            .simulation-frame .target-info-panel {{
                display: flex;
                flex-direction: column;
                align-items: flex-start;
                gap: 6px;
                min-width: 120px;
            }}
            .simulation-frame .frame-canvas-container h5 {{
                font-size: 0.9em;
                color: #666;
                margin-bottom: 4px;
                font-weight: 500;
                display: flex;
                align-items: center;
                justify-content: center;
                gap: 8px;
                position: relative;
            }}
            .simulation-frame canvas {{
                border: 1px solid #c8d1d9;
                cursor: pointer;
                background-color: #f8f9fa;
                max-width: 100%;
                height: auto;
                border-radius: 6px;
                display: block;
                margin: 0 auto;
            }}
            .simulation-frame .boundary-info {{
                font-family: monospace;
                font-size: 0.8em;
                color: #5f6b7a;
                margin-top: 5px;
                text-align: center;
            }}
            .simulation-frame .target-badge {{
                position: absolute;
                bottom: 6px;
                left: 6px;
                background: rgba(255, 255, 255, 0.95);
                border: 1px solid #c8d1d9;
                padding: 4px 8px;
                border-radius: 4px;
                font-size: 0.75em;
                color: #495057;
                font-family: "Segoe UI", system-ui, sans-serif;
                box-shadow: 0 1px 4px rgba(0, 0, 0, 0.08);
            }}

            /* --- Right Column (Editor) --- */
            #editor-column {{
                flex: 1;
                display: flex;
                flex-direction: column;
                padding: 15px;
                border: 1px solid #d1d9e0;
                border-radius: 8px;
                background-color: #ffffff;
                overflow: hidden;
                box-shadow: 0 2px 5px rgba(0, 0, 0, 0.05);
            }}
            #editor-column .editor-controls {{
                display: flex;
                justify-content: space-between;
                align-items: center;
                margin-bottom: 1px;
                flex-shrink: 0;
            }}
            #editor-column label {{
                width: 340px;
                margin-bottom: 0;
                font-weight: 600;
                color: #333;
                font-size: 1em;
            }}
            .bi {{
                vertical-align: -0.125em;
                margin-right: 0.25em;
            }}
            #simulationConfigList {{
                flex-grow: 1;
                width: 100%;
                font-family: "Courier New", Courier, monospace;
                font-size: 0.9em;
                border: 1px solid #ddd;
                resize: none;
                background-color: #f9fafb;
                white-space: pre;
                overflow-wrap: normal;
                overflow: auto;
                padding: 12px;
                border-radius: 6px;
                color: #333;
                line-height: 1.4;
            }}

            .editor-btn-group {{
                display: flex;
                justify-content: flex-end;
                align-items: center;
                gap: 10px;
                width: 100%;
            }}
        </style>
    </head>
    <body>
        <h2>Multi-Robot Simulation Editor</h2>

        <div class="main-container">
            <div id="simulator-column"></div>

            <div class="resize-handle" id="resizeHandle">
                <button
                    class="toggle-editor-btn"
                    id="toggleEditorBtn"
                    title="Toggle editor panel"
                >
                    ◂
                </button>
            </div>

            <div id="editor-column">
                <div class="editor-controls">
                    <label for="simulationConfigList"
                        >Active Simulation Config (JSON):</label
                    >
                    <div class="editor-btn-group">
                        <button
                            id="copyConfigBtn"
                            class="btn btn-secondary btn-sm"
                            title="Copy JSON to clipboard"
                        >
                            <svg
                                xmlns="http://www.w3.org/2000/svg"
                                width="16"
                                height="16"
                                fill="currentColor"
                                class="bi bi-clipboard"
                                viewBox="0 0 16 16"
                            >
                                <path
                                    d="M4 1.5H3a2 2 0 0 0-2 2V14a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V3.5a2 2 0 0 0-2-2h-1v1h1a1 1 0 0 1 1 1V14a1 1 0 0 1-1 1H3a1 1 0 0 1-1-1V3.5a1 1 0 0 1 1-1h1v-1z"
                                />
                                <path
                                    d="M9.5 1a.5.5 0 0 1 .5.5v1a.5.5 0 0 1-.5.5h-3a.5.5 0 0 1-.5-.5v-1a.5.5 0 0 1 .5-.5h3zm-3-1A1.5 1.5 0 0 0 5 1.5v1A1.5 1.5 0 0 0 6.5 4h3A1.5 1.5 0 0 0 11 2.5v-1A1.5 1.5 0 0 0 9.5 0h-3z"
                                />
                            </svg>
                            Copy
                        </button>
                    </div>
                </div>
                <textarea
                    id="simulationConfigList"
                    rows="20"
                    spellcheck="false"
                    readonly
                ></textarea>
            </div>
        </div>

        <script>
            // Inject the JSON data directly as JavaScript object/array literals
            let defaultSimulationConfigList = {};
            let defaultGoalsTargetsResults = {};

            // Assign the injected objects directly, no JSON.parse() needed
            let activeGoalsTargetsResults = defaultGoalsTargetsResults;
            let activeSimulationConfigs = (() => {{
                try {{
                    // Use the injected object directly
                    const parsedData = defaultSimulationConfigList;
                    if (
                        parsedData &&
                        parsedData.simulation_configs &&
                        Array.isArray(parsedData.simulation_configs)
                    ) {{
                        return parsedData.simulation_configs.map((config, index) => {{
                            const boundaries = config.boundary || [-6, 6, -6, 6];
                            const walls = [];

                            // Convert wall array [vertical, horizontal] to wall objects
                            if (config.wall && config.wall.length >= 2) {{
                                const [verticalWall, horizontalWall] = config.wall;

                                if (verticalWall !== null) {{
                                    walls.push({{
                                        type: "vertical",
                                        x1: verticalWall,
                                        y1: boundaries[2] - 5, // ymin - 5
                                        x2: verticalWall,
                                        y2: boundaries[3] + 5, // ymax + 5
                                    }});
                                }}

                                if (horizontalWall !== null) {{
                                    walls.push({{
                                        type: "horizontal",
                                        x1: boundaries[0] - 5, // xmin - 5
                                        y1: horizontalWall,
                                        x2: boundaries[1] + 5, // xmax + 5
                                        y2: horizontalWall,
                                    }});
                                }}
                            }}

                            // Convert targets to grids format
                            const grids = [config.initial_positions || []];
                            const gridSteps = [0]; // Starting position
                            const gridWaypoints = [[]]; // Starting grid has no waypoints
                            const gridExclusivePoints = [[]]; // Starting grid has no exclusive points

                            if (config.targets && Array.isArray(config.targets)) {{
                                config.targets.forEach((target) => {{
                                    if (Array.isArray(target) && target.length >= 4) {{
                                        const [steps, robots, exclusive, waypoints] = target;
                                        grids.push(robots || []);
                                        gridSteps.push(steps || 5);
                                        gridWaypoints.push(waypoints || []);
                                        gridExclusivePoints.push(exclusive || []);
                                    }}
                                }});
                            }}

                            // Ensure at least one target grid
                            if (grids.length < 2) {{
                                grids.push([]);
                                gridSteps.push(5);
                                gridWaypoints.push([]);
                                gridExclusivePoints.push([]);
                            }}

                            // Calculate total steps
                            const totalSteps = gridSteps
                                .slice(1)
                                .reduce((sum, steps) => sum + steps, 0);

                            return {{
                                id: `cfg-default-${{Date.now()}}-${{index}}`,
                                grids,
                                steps: totalSteps,
                                gridSteps,
                                gridWaypoints,
                                gridExclusivePoints,
                                boundaries: {{
                                    xmin: boundaries[0],
                                    xmax: boundaries[1],
                                    ymin: boundaries[2],
                                    ymax: boundaries[3],
                                }},
                                walls,
                                needsRedraw: true,
                            }};
                        }});
                    }} else {{
                        throw new Error(
                            "Invalid JSON format. Expected object with 'simulation_configs' array."
                        );
                    }}
                }} catch (error) {{
                    console.error("Error parsing default configurations:", error);
                    // Fallback to empty array
                    return [];
                }}
            }})();
            // --- Initial Data ---
            /* let activeSimulationConfigs = [
                {{
                    id: `cfg-${{Date.now()}}-0`,
                    grids: [
                        [
                            ["O", 0, 0],
                            ["F", 0, 1],
                            ["R", -1, 1],
                        ],
                        [
                            ["O", 0, 0],
                            ["R", -2, 1],
                            ["F", -1, 1],
                        ],
                    ],
                    steps: 1,
                    boundaries: {{ xmin: -3, xmax: 1, ymin: -1, ymax: 2 }},
                    waypoints: [],
                    exclusive_points: [],
                }},
                {{
                    id: `cfg-${{Date.now()}}-1`,
                    grids: [
                        [
                            ["O", 0, 0],
                            ["F", 0, 1],
                            ["L", -1, 1],
                        ],
                        [
                            ["O", 0, 0],
                            ["L", -2, 1],
                            ["F", -1, 1],
                        ],
                    ],
                    steps: 1,
                    boundaries: {{ xmin: -3, xmax: 1, ymin: -1, ymax: 2 }},
                    waypoints: [],
                    exclusive_points: [],
                }},
            ].map((cfg) => ({{ ...cfg, needsRedraw: true }}));
            */
            // --- Constants and Config ---
            const FRAME_CANVAS_SIZE = 300;
            const FRAME_SCALE = 25;
            const FRAME_OFFSET = FRAME_CANVAS_SIZE / 2;
            const CANVAS_CENTER_X = FRAME_OFFSET;
            const CANVAS_CENTER_Y = FRAME_OFFSET;
            const simConfig = {{
                colors: {{
                    L: 'red', R: 'green', F: 'blue', O: 'orange',
                    r: 'red', g: 'green', b: 'blue', o: 'orange',
                    B: 'blue', G: 'green', Y: 'yellow', P: 'purple',
                    y: 'yellow', p: 'purple', W: '#333', w: '#333'
                }},
                boundaryColor: "#666",
                boundaryWidth: 2,
                robotRadius: 8,
                waypointColor: "#0c2c67",
                exclusivePointColor: "#0c2c67",
                waypointRadius: 8,
                exclusivePointRadius: 8,
            }};

            // --- Global State Variables ---
            let frameElements = {{}};

            // --- Element References ---
            const simulatorColumn = document.getElementById("simulator-column");
            const simulationConfigListTextArea = document.getElementById(
                "simulationConfigList"
            );
            const resizeHandle = document.getElementById("resizeHandle");
            const toggleEditorBtn = document.getElementById("toggleEditorBtn");
            const editorColumn = document.getElementById("editor-column");
            const mainContainer = document.querySelector(".main-container");

            // --- Resize and Toggle Variables ---
            let isResizing = false;
            let isEditorHidden = false;
            const LAYOUT_STORAGE_KEY = "multiRobotSimLayout_v1";

            // --- Helper Functions ---
            function frameWorldToCanvas(x, y) {{
                return {{
                    cx: FRAME_OFFSET + x * FRAME_SCALE,
                    cy: FRAME_OFFSET - y * FRAME_SCALE,
                }};
            }}
            function isWithinBoundaries(x, y, frameBoundaries) {{
                if (!frameBoundaries) return false;
                return (
                    x >= frameBoundaries.xmin &&
                    x <= frameBoundaries.xmax &&
                    y >= frameBoundaries.ymin &&
                    y <= frameBoundaries.ymax
                );
            }}

            // --- Drawing Functions ---
            function drawGridLines(ctx) {{
                if (!ctx) return;
                ctx.clearRect(0, 0, FRAME_CANVAS_SIZE, FRAME_CANVAS_SIZE);
                ctx.strokeStyle = "#e8e8e8";
                ctx.lineWidth = 1;
                ctx.setLineDash([]);
                const gridRange = 6;
                for (let i = -gridRange; i <= gridRange; i++) {{
                    if (i === 0) continue;
                    const {{ cx }} = frameWorldToCanvas(i, 0);
                    ctx.beginPath();
                    ctx.moveTo(cx + 0.5, 0);
                    ctx.lineTo(cx + 0.5, FRAME_CANVAS_SIZE);
                    ctx.stroke();
                    const {{ cy }} = frameWorldToCanvas(0, i);
                    ctx.beginPath();
                    ctx.moveTo(0, cy + 0.5);
                    ctx.lineTo(FRAME_CANVAS_SIZE, cy + 0.5);
                    ctx.stroke();
                }}
                ctx.strokeStyle = "#b0b0b0";
                ctx.lineWidth = 1;
                ctx.beginPath();
                ctx.moveTo(0, FRAME_OFFSET + 0.5);
                ctx.lineTo(FRAME_CANVAS_SIZE, FRAME_OFFSET + 0.5);
                ctx.stroke();
                ctx.beginPath();
                ctx.moveTo(FRAME_OFFSET + 0.5, 0);
                ctx.lineTo(FRAME_OFFSET + 0.5, FRAME_CANVAS_SIZE);
                ctx.stroke();
            }}
            function drawBoundaries(ctx, frameBoundaries, rotation = 0) {{
                if (!ctx || !frameBoundaries) return;

                ctx.save();
                // Apply rotation if specified
                if (rotation) {{
                    ctx.translate(CANVAS_CENTER_X, CANVAS_CENTER_Y);
                    ctx.rotate((rotation * Math.PI) / 180);
                    ctx.translate(-CANVAS_CENTER_X, -CANVAS_CENTER_Y);
                }}

                ctx.strokeStyle = simConfig.boundaryColor;
                ctx.lineWidth = simConfig.boundaryWidth;
                ctx.setLineDash([5, 3]);
                const {{ cx: xmin_cx }} = frameWorldToCanvas(frameBoundaries.xmin, 0);
                const {{ cx: xmax_cx }} = frameWorldToCanvas(frameBoundaries.xmax, 0);
                const {{ cy: ymin_cy }} = frameWorldToCanvas(0, frameBoundaries.ymin);
                const {{ cy: ymax_cy }} = frameWorldToCanvas(0, frameBoundaries.ymax);
                ctx.strokeRect(xmin_cx, ymax_cy, xmax_cx - xmin_cx, ymin_cy - ymax_cy);
                ctx.setLineDash([]);

                ctx.restore();
            }}
            function drawRobots(ctx, robots, frameBoundaries, rotation = 0) {{
                if (!ctx || !robots) return;

                ctx.save();
                // Apply rotation if specified
                if (rotation) {{
                    ctx.translate(CANVAS_CENTER_X, CANVAS_CENTER_Y);
                    ctx.rotate((rotation * Math.PI) / 180);
                    ctx.translate(-CANVAS_CENTER_X, -CANVAS_CENTER_Y);
                }}

                robots.forEach(([color, x, y]) => {{
                    const {{ cx, cy }} = frameWorldToCanvas(x, y);
                    const isInBounds = isWithinBoundaries(x, y, frameBoundaries);
                    ctx.fillStyle = simConfig.colors[color] || "gray";
                    ctx.beginPath();
                    ctx.arc(cx, cy, simConfig.robotRadius, 0, Math.PI * 2);
                    ctx.fill();
                    ctx.strokeStyle = isInBounds ? "#333" : "#e53935";
                    ctx.lineWidth = isInBounds ? 1.5 : 2;
                    ctx.stroke();
                }});

                ctx.restore();
            }}

            function drawWaypoints(ctx, waypoints, frameBoundaries, rotation = 0) {{
                if (!ctx || !waypoints) return;

                ctx.save();
                // Apply rotation if specified
                if (rotation) {{
                    ctx.translate(CANVAS_CENTER_X, CANVAS_CENTER_Y);
                    ctx.rotate((rotation * Math.PI) / 180);
                    ctx.translate(-CANVAS_CENTER_X, -CANVAS_CENTER_Y);
                }}

                waypoints.forEach(([x, y]) => {{
                    const {{ cx, cy }} = frameWorldToCanvas(x, y);
                    const isInBounds = isWithinBoundaries(x, y, frameBoundaries);

                    // Draw circle background
                    const size = simConfig.waypointRadius;
                    ctx.fillStyle = simConfig.waypointColor;
                    ctx.beginPath();
                    ctx.arc(cx, cy, size, 0, Math.PI * 2);
                    ctx.fill();

                    // Draw location pin icon
                    ctx.fillStyle = "#fff";
                    const pinSize = size * 1.3;
                    const pinTop = cy - pinSize * 0.4;
                    const pinRadius = pinSize * 0.4;

                    // Pin head
                    ctx.beginPath();
                    ctx.arc(cx, pinTop, pinRadius, 0, Math.PI * 2);
                    ctx.fill();

                    // Pin point
                    ctx.beginPath();
                    ctx.moveTo(cx - pinRadius * 0.3, pinTop + pinRadius * 0.7);
                    ctx.lineTo(cx, cy + pinSize * 0.3);
                    ctx.lineTo(cx + pinRadius * 0.3, pinTop + pinRadius * 0.7);
                    ctx.closePath();
                    ctx.fill();

                    // Inner dot
                    ctx.fillStyle = simConfig.waypointColor;
                    ctx.beginPath();
                    ctx.arc(cx, pinTop, pinRadius * 0.4, 0, Math.PI * 2);
                    ctx.fill();

                    // Border
                    ctx.strokeStyle = isInBounds ? "#333" : "#e53935";
                    ctx.lineWidth = isInBounds ? 1.5 : 2;
                    ctx.beginPath();
                    ctx.arc(cx, cy, size, 0, Math.PI * 2);
                    ctx.stroke();
                }});

                ctx.restore();
            }}

            function drawExclusivePoints(
                ctx,
                exclusivePoints,
                frameBoundaries,
                rotation = 0
            ) {{
                if (!ctx || !exclusivePoints) return;

                ctx.save();
                // Apply rotation if specified
                if (rotation) {{
                    ctx.translate(CANVAS_CENTER_X, CANVAS_CENTER_Y);
                    ctx.rotate((rotation * Math.PI) / 180);
                    ctx.translate(-CANVAS_CENTER_X, -CANVAS_CENTER_Y);
                }}

                exclusivePoints.forEach(([x, y]) => {{
                    const {{ cx, cy }} = frameWorldToCanvas(x, y);
                    const isInBounds = isWithinBoundaries(x, y, frameBoundaries);

                    // Draw circle background
                    const size = simConfig.exclusivePointRadius;
                    ctx.fillStyle = simConfig.exclusivePointColor;
                    ctx.beginPath();
                    ctx.arc(cx, cy, size, 0, Math.PI * 2);
                    ctx.fill();

                    // Draw X inside
                    ctx.strokeStyle = "#fff";
                    ctx.lineWidth = 3;
                    ctx.beginPath();
                    ctx.moveTo(cx - size * 0.6, cy - size * 0.6);
                    ctx.lineTo(cx + size * 0.6, cy + size * 0.6);
                    ctx.moveTo(cx + size * 0.6, cy - size * 0.6);
                    ctx.lineTo(cx - size * 0.6, cy + size * 0.6);
                    ctx.stroke();

                    // Border
                    ctx.strokeStyle = isInBounds ? "#333" : "#e53935";
                    ctx.lineWidth = isInBounds ? 1.5 : 2;
                    ctx.beginPath();
                    ctx.arc(cx, cy, size, 0, Math.PI * 2);
                    ctx.stroke();
                }});

                ctx.restore();
            }}

            function drawWalls(ctx, walls, frameBoundaries, rotation = 0) {{
                if (!ctx || !walls) return;

                ctx.save();
                // Apply rotation if specified
                if (rotation) {{
                    ctx.translate(FRAME_OFFSET, FRAME_OFFSET);
                    ctx.rotate((rotation * Math.PI) / 180);
                    ctx.translate(-FRAME_OFFSET, -FRAME_OFFSET);
                }}

                // Find all intersection points
                const intersections = [];
                for (let i = 0; i < walls.length; i++) {{
                    for (let j = i + 1; j < walls.length; j++) {{
                        const wall1 = walls[i];
                        const wall2 = walls[j];

                        const intersection = findWallIntersection(wall1, wall2);
                        if (intersection) {{
                            intersections.push({{
                                x: intersection.x,
                                y: intersection.y,
                                wall1: wall1,
                                wall2: wall2,
                            }});
                        }}
                    }}
                }}

                // Draw each wall, trimming at intersections
                walls.forEach((wall) => {{
                    // Find intersections that affect this wall
                    const wallIntersections = intersections.filter(
                        (intersection) =>
                            intersection.wall1 === wall || intersection.wall2 === wall
                    );

                    if (wallIntersections.length === 0) {{
                        // No intersections, draw the full wall
                        const {{ cx: cx1, cy: cy1 }} = frameWorldToCanvas(wall.x1, wall.y1);
                        const {{ cx: cx2, cy: cy2 }} = frameWorldToCanvas(wall.x2, wall.y2);

                        ctx.strokeStyle = "#000";
                        ctx.lineWidth = 4;
                        ctx.lineCap = "butt";
                        ctx.beginPath();
                        ctx.moveTo(cx1, cy1);
                        ctx.lineTo(cx2, cy2);
                        ctx.stroke();
                    }} else {{
                        // Find the closest intersection point to trim the wall
                        let startPoint = {{ x: wall.x1, y: wall.y1 }};
                        let endPoint = {{ x: wall.x2, y: wall.y2 }};

                        // For each intersection, determine if it should replace start or end point
                        wallIntersections.forEach((intersection) => {{
                            const intPoint = {{ x: intersection.x, y: intersection.y }};

                            // Check if intersection is closer to start or end
                            const distToStart =
                                Math.abs(intPoint.x - wall.x1) + Math.abs(intPoint.y - wall.y1);
                            const distToEnd =
                                Math.abs(intPoint.x - wall.x2) + Math.abs(intPoint.y - wall.y2);

                            if (distToStart < distToEnd) {{
                                // Intersection is closer to start, so trim from start
                                if (wall.y1 === wall.y2) {{
                                    // Horizontal wall - check if intersection is between start and end
                                    if (
                                        intPoint.x >= Math.min(wall.x1, wall.x2) &&
                                        intPoint.x <= Math.max(wall.x1, wall.x2)
                                    ) {{
                                        startPoint = intPoint;
                                    }}
                                }} else {{
                                    // Vertical wall - check if intersection is between start and end
                                    if (
                                        intPoint.y >= Math.min(wall.y1, wall.y2) &&
                                        intPoint.y <= Math.max(wall.y1, wall.y2)
                                    ) {{
                                        startPoint = intPoint;
                                    }}
                                }}
                            }} else {{
                                // Intersection is closer to end, so trim from end
                                if (wall.y1 === wall.y2) {{
                                    // Horizontal wall
                                    if (
                                        intPoint.x >= Math.min(wall.x1, wall.x2) &&
                                        intPoint.x <= Math.max(wall.x1, wall.x2)
                                    ) {{
                                        endPoint = intPoint;
                                    }}
                                }} else {{
                                    // Vertical wall
                                    if (
                                        intPoint.y >= Math.min(wall.y1, wall.y2) &&
                                        intPoint.y <= Math.max(wall.y1, wall.y2)
                                    ) {{
                                        endPoint = intPoint;
                                    }}
                                }}
                            }}
                        }});

                        // Draw the trimmed wall
                        const {{ cx: cx1, cy: cy1 }} = frameWorldToCanvas(
                            startPoint.x,
                            startPoint.y
                        );
                        const {{ cx: cx2, cy: cy2 }} = frameWorldToCanvas(
                            endPoint.x,
                            endPoint.y
                        );

                        ctx.strokeStyle = "#000";
                        ctx.lineWidth = 4;
                        ctx.lineCap = "butt";
                        ctx.beginPath();
                        ctx.moveTo(cx1, cy1);
                        ctx.lineTo(cx2, cy2);
                        ctx.stroke();
                    }}
                }});

                // Draw small squares at intersection points for better corner visualization
                intersections.forEach((intersection) => {{
                    const {{ cx, cy }} = frameWorldToCanvas(intersection.x, intersection.y);

                    ctx.fillStyle = "#000";
                    ctx.fillRect(cx - 2, cy - 2, 4, 4);
                }});

                ctx.restore();
            }}

            // Helper function to find intersection between two walls
            function findWallIntersection(wall1, wall2) {{
                const {{ x1: x1a, y1: y1a, x2: x2a, y2: y2a }} = wall1;
                const {{ x1: x1b, y1: y1b, x2: x2b, y2: y2b }} = wall2;

                // Check if wall1 is horizontal and wall2 is vertical
                if (y1a === y2a && x1b === x2b) {{
                    const wallY = y1a;
                    const wallX = x1b;

                    // Check if intersection point is within both wall segments
                    if (
                        wallX >= Math.min(x1a, x2a) &&
                        wallX <= Math.max(x1a, x2a) &&
                        wallY >= Math.min(y1b, y2b) &&
                        wallY <= Math.max(y1b, y2b)
                    ) {{
                        return {{ x: wallX, y: wallY }};
                    }}
                }}

                // Check if wall1 is vertical and wall2 is horizontal
                if (x1a === x2a && y1b === y2b) {{
                    const wallX = x1a;
                    const wallY = y1b;

                    // Check if intersection point is within both wall segments
                    if (
                        wallX >= Math.min(x1b, x2b) &&
                        wallX <= Math.max(x1b, x2b) &&
                        wallY >= Math.min(y1a, y2a) &&
                        wallY <= Math.max(y1a, y2a)
                    ) {{
                        return {{ x: wallX, y: wallY }};
                    }}
                }}

                return null;
            }}

            // --- Core Frame Rendering and Updating ---
            function redrawFrame(configId) {{
                const frameData = frameElements[configId];
                const config = activeSimulationConfigs.find((c) => c.id === configId);
                if (!frameData || !config) return;

                const boundaries = config.boundaries || {{
                    xmin: -6,
                    xmax: 6,
                    ymin: -6,
                    ymax: 6,
                }};
                const rotation = config.rotation || 0;

                // Redraw each canvas for the frame
                frameData.canvases.forEach((canvas, gridIndex) => {{
                    const ctx = canvas.getContext("2d");
                    const robots = config.grids[gridIndex] || [];
                    const gridWaypoints =
                        (config.gridWaypoints && config.gridWaypoints[gridIndex]) || [];
                    const gridExclusivePoints =
                        (config.gridExclusivePoints &&
                            config.gridExclusivePoints[gridIndex]) ||
                        [];

                    drawGridLines(ctx);
                    drawBoundaries(ctx, boundaries, rotation);
                    drawWaypoints(ctx, gridWaypoints, boundaries, rotation);
                    drawExclusivePoints(ctx, gridExclusivePoints, boundaries, rotation);
                    drawWalls(ctx, config.walls || [], boundaries, rotation);
                    drawRobots(ctx, robots, boundaries, rotation);
                }});

                // Update boundary info text
                const boundsInfo = frameData.frameDiv.querySelector(".boundary-info");
                if (boundsInfo) {{
                    boundsInfo.querySelector("span.xmin").textContent = boundaries.xmin;
                    boundsInfo.querySelector("span.xmax").textContent = boundaries.xmax;
                    boundsInfo.querySelector("span.ymin").textContent = boundaries.ymin;
                    boundsInfo.querySelector("span.ymax").textContent = boundaries.ymax;
                }}
                if (frameData.stepsInput) {{
                    frameData.stepsInput.value = config.steps;
                }}
                config.needsRedraw = false;
            }}

            function createFrameElement(configData, index) {{
                const frameDiv = document.createElement("div");
                frameDiv.className = "simulation-frame";
                frameDiv.dataset.configId = configData.id;

                // --- Header ---
                const headerDiv = document.createElement("div");
                headerDiv.className = "frame-header";
                const title = document.createElement("h4");

                // Calculate total steps for the title
                let totalSteps = 0;
                if (configData.gridSteps && configData.gridSteps.length > 1) {{
                    totalSteps = configData.gridSteps
                        .slice(1)
                        .reduce((sum, steps) => sum + steps, 0);
                }} else if (configData.steps) {{
                    totalSteps = configData.steps;
                }}
                title.textContent = `Simulation ${{
                    index + 1
                }} (Total Steps: ${{totalSteps}})`;

                headerDiv.append(title);
                frameDiv.appendChild(headerDiv);

                // --- Grids Wrapper ---
                const gridsWrapper = document.createElement("div");
                gridsWrapper.className = "grids-wrapper";
                frameDiv.appendChild(gridsWrapper);

                // --- Canvas Creation ---
                const boundaries = configData.boundaries || {{
                    xmin: -6,
                    xmax: 6,
                    ymin: -6,
                    ymax: 6,
                }};
                const canvases = [];

                (configData.grids || []).forEach((grid, gridIndex) => {{
                    const container = document.createElement("div");
                    container.className = "frame-canvas-container";
                    container.dataset.gridIndex = gridIndex;
                    container.dataset.configId = configData.id;

                    // Create title with proper naming
                    const titleDiv = document.createElement("h5");
                    if (gridIndex === 0) {{
                        titleDiv.textContent = "Starting Position";
                    }} else {{
                        titleDiv.textContent = `Target ${{gridIndex}}`;
                    }}
                    container.appendChild(titleDiv);

                    const canvas = document.createElement("canvas");
                    canvas.id = `canvas-${{configData.id}}-${{gridIndex}}`;
                    canvas.width = FRAME_CANVAS_SIZE;
                    canvas.height = FRAME_CANVAS_SIZE;
                    canvas.dataset.gridIndex = gridIndex; // Store index
                    container.appendChild(canvas);
                    canvases.push(canvas);

                    // Add boundary info only to the first canvas
                    if (gridIndex === 0) {{
                        const boundsInfo = document.createElement("div");
                        boundsInfo.className = "boundary-info";
                        boundsInfo.innerHTML = `Bounds: x:[<span class="xmin">${{boundaries.xmin}}</span>,<span class="xmax">${{boundaries.xmax}}</span>] y:[<span class="ymin">${{boundaries.ymin}}</span>,<span class="ymax">${{boundaries.ymax}}</span>]`;
                        container.appendChild(boundsInfo);
                    }} else {{
                        // New layout: left label (Target N, steps), right: executions + view btn
                        const infoRow = document.createElement("div");
                        infoRow.className = "target-info-row";
                        infoRow.style.cssText = `
                            display: flex;
                            align-items: center;
                            justify-content: space-between;
                            margin-top: 5px;
                            font-family: monospace;
                            font-size: 0.8em;
                            color: #5f6b7a;
                            gap: 10px;
                        `;

                        // Left: label
                        const leftLabel = document.createElement("span");
                        const steps =
                            (configData.gridSteps && configData.gridSteps[gridIndex]) || 5;
                        leftLabel.textContent = ` (${{steps}} steps)`;
                        leftLabel.style.cssText = "font-weight: 500;";

                        // Right: executions + view btn
                        const rightBox = document.createElement("span");
                        rightBox.style.cssText =
                            "display: flex; align-items: center; gap: 8px;";
                        // Placeholder for executions count
                        const execCount = document.createElement("span");
                        execCount.textContent = "Executions: 0";
                        execCount.className = "exec-count";
                        execCount.style.cssText = "font-size: 1em; font-weight: bold;";
                        // View button
                        const viewBtn = document.createElement("button");
                        viewBtn.textContent = "View";
                        viewBtn.className = "view-exec-btn";
                        viewBtn.style.cssText = `
                            padding: 4px 12px;
                            font-size: 1em;
                            border: 1px solid #007bff;
                            border-radius: 4px;
                            background: #007bff;
                            cursor: pointer;
                            color: white;
                            font-weight: bold;
                        `;
                        viewBtn.title = "View execution results";
                        const goalIndex = index;
                        const targetIndex = gridIndex - 1;
                        const resultData = activeGoalsTargetsResults[goalIndex]?.[targetIndex];
                        if (resultData && resultData.execution_count > 0 && resultData.result_path) {{
                            execCount.textContent = `Executions: ${{resultData.execution_count}}`;
                            viewBtn.addEventListener("click", (e) => {{
                                e.preventDefault();
                                window.open(resultData.result_path, '_blank');
                            }});
                        }} else {{
                            execCount.textContent = "Executions: 0";
                            viewBtn.disabled = true;
                            viewBtn.style.opacity = 0.5;
                            viewBtn.style.background = "#6c757d";
                            viewBtn.style.borderColor = "#6c757d";
                        }}
                        rightBox.appendChild(execCount);
                        rightBox.appendChild(viewBtn);

                        infoRow.appendChild(leftLabel);
                        infoRow.appendChild(rightBox);
                        container.appendChild(infoRow);
                    }}
                    gridsWrapper.appendChild(container);
                }});

                // --- Store References ---
                frameElements[configData.id] = {{
                    frameDiv,
                    canvases,
                    titleElement: title,
                }};

                return frameDiv;
            }}

            function renderAllFrames() {{
                // Store scroll positions of grids wrappers before re-rendering
                const scrollPositions = {{}};
                const existingFrames =
                    simulatorColumn.querySelectorAll(".simulation-frame");
                existingFrames.forEach((frame) => {{
                    const configId = frame.dataset.configId;
                    const gridsWrapper = frame.querySelector(".grids-wrapper");
                    if (gridsWrapper && configId) {{
                        scrollPositions[configId] = gridsWrapper.scrollLeft;
                    }}
                }});

                // Clear existing frames
                const framesToRemove =
                    simulatorColumn.querySelectorAll(".simulation-frame");
                framesToRemove.forEach((frame) => simulatorColumn.removeChild(frame));
                frameElements = {{}};

                // Create and append frames
                activeSimulationConfigs.forEach((config, index) => {{
                    const frameDiv = createFrameElement(config, index);
                    simulatorColumn.appendChild(frameDiv);
                    redrawFrame(config.id);

                    // Restore scroll position for this frame's grids wrapper
                    if (scrollPositions[config.id] !== undefined) {{
                        const gridsWrapper = frameDiv.querySelector(".grids-wrapper");
                        if (gridsWrapper) {{
                            // Use setTimeout to ensure DOM is fully rendered before setting scroll
                            setTimeout(() => {{
                                gridsWrapper.scrollLeft = scrollPositions[config.id];
                            }}, 0);
                        }}
                    }}
                }});

                updateFullConfigTextArea();
                // saveConfigsToLocalStorage(); // Not needed for display-only viewer
            }}

            function updateGridSteps(configId, gridIndex, newValue) {{
                const config = activeSimulationConfigs.find(
                    (cfg) => cfg.id === configId
                );
                if (config) {{
                    // Initialize gridSteps if it doesn't exist
                    if (!config.gridSteps) {{
                        config.gridSteps = Array(config.grids.length).fill(0);
                        config.gridSteps[1] = 5; // Default for first target
                    }}

                    // Update the specific grid's steps
                    config.gridSteps[gridIndex] = Math.max(0, newValue);

                    // Calculate total steps (sum of all grid steps except starting position)
                    const totalSteps = config.gridSteps
                        .slice(1)
                        .reduce((sum, steps) => sum + steps, 0);
                    config.steps = totalSteps;

                    // Update frame title to show total steps
                    updateFrameTitle(configId);

                    updateFullConfigTextArea();
                    // saveConfigsToLocalStorage(); // Not needed for display-only viewer
                }}
            }}

            function updateFrameTitle(configId) {{
                const config = activeSimulationConfigs.find(
                    (cfg) => cfg.id === configId
                );
                if (config && frameElements[configId]?.titleElement) {{
                    const frameIndex =
                        activeSimulationConfigs.findIndex((cfg) => cfg.id === configId) + 1;
                    const totalSteps = config.steps || 0;
                    frameElements[
                        configId
                    ].titleElement.textContent = `Simulation ${{frameIndex}} (Total Steps: ${{totalSteps}})`;
                }}
            }}

            // --- Editor Display (Shows Full List, Read-Only) ---
            function updateFullConfigTextArea() {{
                if (!simulationConfigListTextArea) return;
                if (activeSimulationConfigs.length === 0) {{
                    simulationConfigListTextArea.value = JSON.stringify(
                        {{
                            simulation_configs: [],
                        }},
                        null,
                        2
                    );
                    return;
                }}

                try {{
                    // Generate JSON format (existing logic)
                    const simulationConfigs = activeSimulationConfigs.map((config) => {{
                        const boundaries = config.boundaries || {{
                            xmin: -6,
                            xmax: 6,
                            ymin: -6,
                            ymax: 6,
                        }};
                        const grids = config.grids || [];
                        const gridSteps = config.gridSteps || [];
                        const gridWaypoints = config.gridWaypoints || [];
                        const gridExclusivePoints = config.gridExclusivePoints || [];
                        const walls = config.walls || [];

                        // Find vertical and horizontal walls
                        let verticalWall = null,
                            horizontalWall = null;
                        walls.forEach((wall) => {{
                            if (wall.type === "vertical") {{
                                verticalWall = wall.x1; // x position of vertical wall
                            }} else if (wall.type === "horizontal") {{
                                horizontalWall = wall.y1; // y position of horizontal wall
                            }}
                        }});

                        // Format grids: starting grid as initial_positions, targets with steps, exclusive points, and waypoints
                        let initialPositions = [];
                        let targets = [];

                        if (grids.length > 0) {{
                            // Starting position (first grid)
                            initialPositions = grids[0].map(([color, x, y]) => [color, x, y]);

                            // Target grids with steps, exclusive points, and waypoints (remaining grids)
                            for (let i = 1; i < grids.length; i++) {{
                                const steps = (gridSteps && gridSteps[i]) || 5;
                                const robots = grids[i].map(([color, x, y]) => [color, x, y]);
                                const exclusivePoints = (gridExclusivePoints[i] || []).map(
                                    ([x, y]) => [x, y]
                                );
                                const waypoints = (gridWaypoints[i] || []).map(([x, y]) => [
                                    x,
                                    y,
                                ]);
                                targets.push([steps, robots, exclusivePoints, waypoints]);
                            }}
                        }}

                        return {{
                            initial_positions: initialPositions,
                            targets: targets,
                            boundary: [
                                boundaries.xmin,
                                boundaries.xmax,
                                boundaries.ymin,
                                boundaries.ymax,
                            ],
                            wall: [verticalWall, horizontalWall],
                        }};
                    }});

                    const fullConfig = {{
                        simulation_configs: simulationConfigs,
                    }};

                    // Custom JSON stringify to keep position arrays on single lines
                    const customStringify = (obj, indent = 0) => {{
                        const spaces = "  ".repeat(indent);

                        if (Array.isArray(obj)) {{
                            // Check if this is a position array (contains [color, x, y] tuples)
                            const isPositionArray =
                                obj.length > 0 &&
                                obj.every(
                                    (item) =>
                                        Array.isArray(item) &&
                                        item.length === 3 &&
                                        typeof item[0] === "string" &&
                                        typeof item[1] === "number" &&
                                        typeof item[2] === "number"
                                );

                            // Check if this is a boundary array (4 numbers)
                            const isBoundaryArray =
                                obj.length === 4 &&
                                obj.every((item) => typeof item === "number");

                            // Check if this is a wall array (2 values, can be null or number)
                            const isWallArray =
                                obj.length === 2 &&
                                obj.every((item) => item === null || typeof item === "number");

                            if (isPositionArray) {{
                                // Format position arrays on single line
                                return (
                                    "[" +
                                    obj
                                        .map(
                                            (pos) =>
                                                `[${{JSON.stringify(pos[0])}}, ${{pos[1]}}, ${{pos[2]}}]`
                                        )
                                        .join(", ") +
                                    "]"
                                );
                            }} else if (isBoundaryArray || isWallArray) {{
                                // Format boundary and wall arrays on single line
                                return (
                                    "[" +
                                    obj
                                        .map((item) => (item === null ? "null" : item))
                                        .join(", ") +
                                    "]"
                                );
                            }} else {{
                                // Check if this is a targets array with position sub-arrays
                                const isTargetsArray =
                                    obj.length > 0 &&
                                    obj.every(
                                        (item) =>
                                            Array.isArray(item) &&
                                            item.length >= 2 &&
                                            Array.isArray(item[1])
                                    );

                                if (isTargetsArray) {{
                                    // Format targets array with proper indentation but keep position sub-arrays on single lines
                                    const items = obj.map((target) => {{
                                        const [steps, robots, exclusive, waypoints] = target;
                                        const robotsStr = Array.isArray(robots)
                                            ? "[" +
                                                robots
                                                    .map(
                                                        (pos) =>
                                                            `[${{JSON.stringify(pos[0])}}, ${{pos[1]}}, ${{
                                                                pos[2]
                                                            }}]`
                                                    )
                                                    .join(", ") +
                                                "]"
                                            : JSON.stringify(robots);
                                        const exclusiveStr = Array.isArray(exclusive)
                                            ? "[" +
                                                exclusive
                                                    .map((pos) => `[${{pos[0]}}, ${{pos[1]}}]`)
                                                    .join(", ") +
                                                "]"
                                            : JSON.stringify(exclusive);
                                        const waypointsStr = Array.isArray(waypoints)
                                            ? "[" +
                                                waypoints
                                                    .map((pos) => `[${{pos[0]}}, ${{pos[1]}}]`)
                                                    .join(", ") +
                                                "]"
                                            : JSON.stringify(waypoints);

                                        return `${{spaces}}  [${{steps}}, ${{robotsStr}}, ${{exclusiveStr}}, ${{waypointsStr}}]`;
                                    }});
                                    return "[\n" + items.join(",\n") + `\n${{spaces}}]`;
                                }} else {{
                                    // Regular array formatting
                                    const items = obj.map((item) =>
                                        customStringify(item, indent)
                                    );
                                    return (
                                        "[\n" +
                                        items.map((item) => spaces + "  " + item).join(",\n") +
                                        `\n${{spaces}}]`
                                    );
                                }}
                            }}
                        }} else if (typeof obj === "object" && obj !== null) {{
                            const entries = Object.entries(obj);
                            const items = entries.map(([key, value]) => {{
                                const formattedValue = customStringify(value, indent + 1);
                                return `${{spaces}}  "${{key}}": ${{formattedValue}}`;
                            }});
                            return "{{\n" + items.join(",\n") + `\n${{spaces}}}}`;
                        }} else {{
                            return JSON.stringify(obj);
                        }}
                    }};

                    simulationConfigListTextArea.value = customStringify(fullConfig);
                }} catch (error) {{
                    console.error("Error formatting config for text area:", error);
                    simulationConfigListTextArea.value = JSON.stringify(
                        {{
                            error: "Error generating configuration JSON.",
                        }},
                        null,
                        2
                    );
                }}
            }}

            // --- Resize and Toggle Functionality ---
            function initializeResizeAndToggle() {{
                // Resize functionality
                resizeHandle.addEventListener("mousedown", (e) => {{
                    isResizing = true;
                    document.body.style.cursor = "col-resize";
                    document.body.style.userSelect = "none";
                    e.preventDefault();
                }});

                document.addEventListener("mousemove", (e) => {{
                    if (!isResizing || isEditorHidden) return;

                    const containerRect = mainContainer.getBoundingClientRect();
                    const mouseX = e.clientX - containerRect.left;
                    const totalWidth = containerRect.width;

                    // Calculate new widths as percentages
                    const simulatorWidth = Math.max(
                        30,
                        Math.min(70, (mouseX / totalWidth) * 100)
                    );
                    const editorWidth = 100 - simulatorWidth;

                    // Apply new widths
                    simulatorColumn.style.flex = `0 0 ${{simulatorWidth}}%`;
                    editorColumn.style.flex = `0 0 ${{editorWidth}}%`;
                }});

                document.addEventListener("mouseup", () => {{
                    if (isResizing) {{
                        isResizing = false;
                        document.body.style.cursor = "";
                        document.body.style.userSelect = "";
                        // Save layout after resize
                        saveLayoutToLocalStorage();
                    }}
                }});

                // Toggle functionality
                toggleEditorBtn.addEventListener("click", (e) => {{
                    e.stopPropagation();
                    toggleEditor();
                }});

                // Copy config functionality
                const copyConfigBtn = document.getElementById("copyConfigBtn");
                if (copyConfigBtn) {{
                    copyConfigBtn.addEventListener("click", async (e) => {{
                        e.preventDefault();
                        try {{
                            await navigator.clipboard.writeText(
                                simulationConfigListTextArea.value
                            );
                            // Visual feedback
                            const originalText = copyConfigBtn.textContent;
                            copyConfigBtn.textContent = "Copied!";
                            copyConfigBtn.style.backgroundColor = "#28a745";
                            setTimeout(() => {{
                                copyConfigBtn.textContent = originalText;
                                copyConfigBtn.style.backgroundColor = "";
                            }}, 2000);
                        }} catch (err) {{
                            console.error("Failed to copy text: ", err);
                            // Fallback for older browsers
                            simulationConfigListTextArea.select();
                            document.execCommand("copy");
                            const originalText = copyConfigBtn.textContent;
                            copyConfigBtn.textContent = "Copied!";
                            setTimeout(() => {{
                                copyConfigBtn.textContent = originalText;
                            }}, 2000);
                        }}
                    }});
                }}
            }}

            function toggleEditor() {{
                isEditorHidden = !isEditorHidden;

                if (isEditorHidden) {{
                    // Hide editor
                    editorColumn.style.display = "none";
                    resizeHandle.style.cursor = "pointer";
                    simulatorColumn.style.flex = "1";
                    toggleEditorBtn.textContent = "▸";
                    toggleEditorBtn.title = "Show editor panel";
                }} else {{
                    // Show editor
                    editorColumn.style.display = "flex";
                    resizeHandle.style.cursor = "col-resize";
                    simulatorColumn.style.flex = "0 0 50%";
                    editorColumn.style.flex = "0 0 50%";
                    toggleEditorBtn.textContent = "◂";
                    toggleEditorBtn.title = "Hide editor panel";
                }}

                // Save layout state
                saveLayoutToLocalStorage();
            }}

            // --- Layout Storage Functions ---
            function saveLayoutToLocalStorage() {{
                try {{
                    const layout = {{
                        isEditorHidden: isEditorHidden,
                        simulatorWidth: simulatorColumn.style.flex || "0 0 50%",
                        editorWidth: editorColumn.style.flex || "0 0 50%",
                    }};
                    localStorage.setItem(LAYOUT_STORAGE_KEY, JSON.stringify(layout));
                }} catch (e) {{
                    console.error("Failed to save layout:", e);
                }}
            }}

            function loadLayoutFromLocalStorage() {{
                try {{
                    const raw = localStorage.getItem(LAYOUT_STORAGE_KEY);
                    if (!raw) return null;
                    return JSON.parse(raw);
                }} catch (e) {{
                    console.error("Failed to load layout:", e);
                    return null;
                }}
            }}

            function applyLayoutFromStorage() {{
                const layout = loadLayoutFromLocalStorage();
                if (!layout) return;

                // Apply saved layout
                if (layout.simulatorWidth) {{
                    simulatorColumn.style.flex = layout.simulatorWidth;
                }}
                if (layout.editorWidth) {{
                    editorColumn.style.flex = layout.editorWidth;
                }}

                // Apply editor visibility state
                if (layout.isEditorHidden) {{
                    isEditorHidden = false; // Reset state first
                    toggleEditor(); // This will set it to hidden and update UI
                }}
            }}

            // --- Initialization ---
            function initialize() {{
                // For display-only viewer, we don't load cached configs since configs are fixed
                // const cached = loadConfigsFromLocalStorage();
                // if (cached && cached.length > 0) {{
                //    activeSimulationConfigs = cached;
                // }}

                // Migrate old data format if necessary
                activeSimulationConfigs.forEach((config) => {{
                    if (config.startRobots || config.endRobots) {{
                        config.grids = [];
                        if (config.startRobots) config.grids.push(config.startRobots);
                        if (config.endRobots) config.grids.push(config.endRobots);
                        delete config.startRobots;
                        delete config.endRobots;
                    }}
                    // Ensure waypoints and exclusive_points exist
                    if (!config.waypoints) config.waypoints = [];
                    if (!config.exclusive_points) config.exclusive_points = [];
                    if (!config.walls) config.walls = [];

                    // Migrate to new per-grid waypoints/exclusive points format
                    if (!config.gridWaypoints || !config.gridExclusivePoints) {{
                        const gridCount = Math.max(2, (config.grids || []).length);
                        config.gridWaypoints = new Array(gridCount)
                            .fill(null)
                            .map(() => []);
                        config.gridExclusivePoints = new Array(gridCount)
                            .fill(null)
                            .map(() => []);

                        // If there were legacy global waypoints/exclusive_points, distribute them to target grids
                        if (config.waypoints && config.waypoints.length > 0) {{
                            for (let i = 1; i < gridCount; i++) {{
                                config.gridWaypoints[i] = [...config.waypoints];
                            }}
                        }}
                        if (config.exclusive_points && config.exclusive_points.length > 0) {{
                            for (let i = 1; i < gridCount; i++) {{
                                config.gridExclusivePoints[i] = [...config.exclusive_points];
                            }}
                        }}
                    }}

                    // Migrate to new gridSteps format
                    if (!config.gridSteps || config.gridSteps.length === 0) {{
                        config.gridSteps = Array(config.grids.length).fill(0);
                        // Set starting position to 0, and distribute remaining steps among target grids
                        if (config.gridSteps.length > 1) {{
                            const totalSteps = config.steps || 5;
                            const numTargetGrids = config.gridSteps.length - 1;
                            const stepsPerGrid = Math.floor(totalSteps / numTargetGrids);
                            const remainder = totalSteps % numTargetGrids;

                            for (let i = 1; i < config.gridSteps.length; i++) {{
                                config.gridSteps[i] = stepsPerGrid;
                                // Add remainder to the last grid
                                if (i === config.gridSteps.length - 1) {{
                                    config.gridSteps[i] += remainder;
                                }}
                            }}

                            // Update total steps to match calculated gridSteps
                            config.steps = config.gridSteps
                                .slice(1)
                                .reduce((sum, steps) => sum + steps, 0);
                        }}
                    }} else if (config.gridSteps.length !== config.grids.length) {{
                        // Handle case where gridSteps array length doesn't match grids length
                        const oldGridSteps = [...config.gridSteps];
                        config.gridSteps = Array(config.grids.length).fill(0);

                        // Copy existing values where possible
                        for (
                            let i = 0;
                            i < Math.min(oldGridSteps.length, config.gridSteps.length);
                            i++
                        ) {{
                            config.gridSteps[i] = oldGridSteps[i];
                        }}

                        // If we have more grids than gridSteps, distribute remaining steps
                        if (config.grids.length > oldGridSteps.length) {{
                            const totalSteps = config.steps || 5;
                            const currentTotal = config.gridSteps
                                .slice(1)
                                .reduce((sum, steps) => sum + steps, 0);
                            const remainingSteps = Math.max(0, totalSteps - currentTotal);
                            const numNewGrids = config.grids.length - oldGridSteps.length;

                            if (numNewGrids > 0 && remainingSteps > 0) {{
                                const stepsPerNewGrid = Math.floor(
                                    remainingSteps / numNewGrids
                                );
                                const remainder = remainingSteps % numNewGrids;

                                for (
                                    let i = oldGridSteps.length;
                                    i < config.gridSteps.length;
                                    i++
                                ) {{
                                    config.gridSteps[i] = stepsPerNewGrid;
                                    if (i === config.gridSteps.length - 1) {{
                                        config.gridSteps[i] += remainder;
                                    }}
                                }}
                            }}
                        }}
                    }}

                    // Ensure steps is properly calculated from gridSteps
                    if (config.gridSteps && config.gridSteps.length > 1) {{
                        config.steps = config.gridSteps
                            .slice(1)
                            .reduce((sum, steps) => sum + steps, 0);
                    }}
                }});

                initializeResizeAndToggle();
                applyLayoutFromStorage();
                renderAllFrames();

                // Update all frame titles to ensure they show correct total steps
                activeSimulationConfigs.forEach((config) => {{
                    updateFrameTitle(config.id);
                }});
            }}

            document.addEventListener("DOMContentLoaded", initialize);
        </script>
    </body>
</html>"##,
        configs_json, result_json
    )
    // Removed the stray, invalid code that was here
}
