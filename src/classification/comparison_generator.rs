use crate::modules::algorithm_experiments_modules::algorithm_experiments::AlgorithmExperiments;
use crate::modules::classification::algorithm_data::AlgorithmData;
use serde::Serialize;
use std::fs;
use std::path::Path;

pub fn generate_multi_algorithm_viewer(
    experiments: &[AlgorithmExperiments],
    output_path: &str,
    root_name: &str,
) {
    // 1. Prepare output directory
    let output_path_obj = std::path::Path::new(output_path);
    let data_dir = output_path_obj.join("multiview_core_data");
    fs::create_dir_all(&data_dir).expect("Failed to create data directory");

    // 2. Get all signatures
    let all_signatures: Vec<Vec<String>> = experiments
        .iter()
        .map(|exp| exp.get_experiment_signatures())
        .collect();

    if all_signatures.is_empty() || all_signatures[0].is_empty() {
        return;
    }

    let num_experiments = all_signatures[0].len();
    let mut common_experiments_data = Vec::new();

    // 3. Iterate experiments
    for i in 0..num_experiments {
        let mut experiment_executions = Vec::new();

        // Extract common data from the first algorithm
        let first_sig = &all_signatures[0][i];
        let parts: Vec<&str> = first_sig.split('|').collect();
        // parts[0] is GRID:..., parts[1] is INIT:..., parts[2] is ESSENTIAL:...

        let grid_part = parts[0].replace("GRID:", "");
        let init_part = parts[1].replace("INIT:", "");
        let essential_part = parts[2].replace("ESSENTIAL:", "");

        // Parse grid part to get config sig
        let grid_values: Vec<&str> = grid_part.split(',').collect();
        // minX, maxX, minY, maxY, columns, rows, obstacleX, obstacleY
        let columns = grid_values.get(4).unwrap_or(&"?");
        let rows = grid_values.get(5).unwrap_or(&"?");
        let obs_x = grid_values.get(6).unwrap_or(&"?");
        let obs_y = grid_values.get(7).unwrap_or(&"?");

        let grid_config_sig = format!("Col:{}, Row:{} (O,{},{})", columns, rows, obs_x, obs_y);

        common_experiments_data.push(serde_json::json!({
            "id": i,
            "grid_info": parts[0],
            "init_info": parts[1],
            "essential": essential_part == "true",
            "grid_config_sig": grid_config_sig,
            "initial_positions": init_part
        }));

        // Collect executions for all algorithms
        for (algo_idx, algo_sigs) in all_signatures.iter().enumerate() {
            let sig = &algo_sigs[i];
            let parts: Vec<&str> = sig.split('|').collect();
            let history = parts.get(3).unwrap_or(&"");

            // Get cycle_len from the experiment result
            let cycle_len = experiments[algo_idx]
                .experiments
                .get(i)
                .map(|exp| exp.cycle_len)
                .unwrap_or(0);

            experiment_executions.push(serde_json::json!({
                "algo_name": experiments[algo_idx].name,
                "status": experiments[algo_idx].status.simple_label(),
                "history": history,
                "cycle_len": cycle_len
            }));
        }

        // Write experiment file
        let exp_js_content = format!(
            "window.experimentData_{} = {};",
            i,
            serde_json::to_string(&experiment_executions).unwrap()
        );
        fs::write(
            data_dir.join(format!("experiment_{}.js", i)),
            exp_js_content,
        )
        .expect("Failed to write experiment file");
    }

    // 4. Write common data file
    let algorithms_meta: Vec<_> = experiments
        .iter()
        .map(|exp| {
            serde_json::json!({
                "name": exp.name,
                "status": exp.status.simple_label()
            })
        })
        .collect();

    let common_data = serde_json::json!({
        "experiments": common_experiments_data,
        "algorithms": algorithms_meta
    });

    let common_js_content = format!(
        "window.commonData = {};",
        serde_json::to_string(&common_data).unwrap()
    );
    fs::write(data_dir.join("common_data.js"), common_js_content)
        .expect("Failed to write common data file");

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Multi-Algorithm Viewer</title>
    <link href="https://cdn.jsdelivr.net/npm/bootstrap@5.3.2/dist/css/bootstrap.min.css" rel="stylesheet">
    <style>
        body {{
            background: #f5f5f5;
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Arial, sans-serif;
            padding: 15px;
        }}

        .container {{
            max-width: 1600px;
            margin: 0 auto;
        }}

        .header {{
            background: white;
            padding: 15px;
            margin-bottom: 15px;
            border-radius: 6px;
            border: 1px solid #ddd;
        }}

        .header h1 {{
            font-size: 1.5rem;
            margin: 0 0 8px 0;
            color: #333;
        }}

        .experiment-selector {{
            background: white;
            padding: 15px;
            margin-bottom: 15px;
            border-radius: 6px;
            border: 1px solid #ddd;
        }}

        .experiment-selector label {{
            font-weight: 600;
            margin-bottom: 6px;
            display: block;
            color: #333;
            font-size: 0.9rem;
        }}

        select.form-select {{
            width: 100%;
            padding: 8px;
            border: 1px solid #ccc;
            border-radius: 4px;
            font-size: 0.95rem;
        }}

        select.form-select:focus {{
            border-color: #007bff;
            outline: none;
            box-shadow: 0 0 0 0.2rem rgba(0, 123, 255, 0.25);
        }}

        .control-panel {{
            background: white;
            padding: 15px;
            margin-bottom: 15px;
            border-radius: 6px;
            border: 1px solid #ddd;
        }}

        #controlPanel {{
            position: sticky;
            top: 0;
            z-index: 1000;
            box-shadow: 0 4px 10px rgba(0,0,0,0.1);
            border-top: none;
            border-top-left-radius: 0;
            border-top-right-radius: 0;
        }}

        .animation-controls {{
            display: flex;
            gap: 10px;
            align-items: center;
            justify-content: center;
            flex-wrap: wrap;
        }}

        .animation-controls button {{
            padding: 10px 24px;
            border: 1px solid #007bff;
            background: #007bff;
            color: white;
            border-radius: 4px;
            cursor: pointer;
            font-size: 1rem;
            font-weight: 500;
        }}

        .animation-controls button:hover {{
            background: #0056b3;
            border-color: #0056b3;
        }}

        .animation-controls button:disabled {{
            background: #6c757d;
            border-color: #6c757d;
            cursor: not-allowed;
            opacity: 0.6;
        }}

        .animation-controls button.find-diff {{
            background: #dc3545;
            border-color: #dc3545;
        }}

        .animation-controls button.find-diff:hover {{
            background: #c82333;
            border-color: #bd2130;
        }}

        .step-info {{
            font-size: 1.1rem;
            color: #333;
            font-weight: 600;
            margin: 0 20px;
        }}

        .grids-container {{
            margin-bottom: 20px;
        }}

        .algorithm-grid {{
            background: white;
            padding: 15px;
            margin-bottom: 20px;
            border-radius: 6px;
            border: 2px solid #ddd;
            text-align: center;
        }}

        .algorithm-grid.different {{
            border-color: #dc3545;
            background: #fff5f5;
        }}

        .algorithm-grid h3 {{
            font-size: 1.1rem;
            color: #007bff;
            margin: 0 0 10px 0;
        }}

        .grid-canvas {{
            border: 3px solid black;
            margin: 10px auto;
            background: #f8f9fa;
            display: block;
        }}

        .no-selection {{
            background: white;
            text-align: center;
            padding: 60px 20px;
            color: #999;
            border-radius: 6px;
            border: 1px solid #ddd;
        }}

        .no-selection h3 {{
            font-size: 1.4rem;
            margin-bottom: 10px;
        }}

        .filter-toggle-link {{
            color: #007bff;
            cursor: pointer;
            font-size: 0.9rem;
            text-decoration: underline;
            display: inline-block;
            margin-top: 8px;
        }}

        .filter-toggle-link:hover {{
            color: #0056b3;
        }}

        .filter-section {{
            display: none;
            margin-top: 15px;
            padding-top: 15px;
            border-top: 2px solid #e0e0e0;
        }}

        .filter-section.visible {{
            display: block;
        }}

        .filter-grid-list {{
            display: grid;
            grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
            gap: 15px;
            margin-top: 10px;
        }}

        .filter-grid-option {{
            border: 2px solid #ddd;
            border-radius: 6px;
            padding: 10px;
            cursor: pointer;
            text-align: center;
            background: white;
            transition: all 0.2s;
        }}

        .filter-grid-option:hover {{
            border-color: #007bff;
            box-shadow: 0 2px 8px rgba(0, 123, 255, 0.2);
        }}

        .filter-grid-option.selected {{
            border-color: #28a745;
            background: #d4edda;
            box-shadow: 0 2px 8px rgba(40, 167, 69, 0.3);
        }}

        .all-option {{
            background: #e7f3ff;
            font-weight: 600;
            padding: 30px 10px;
        }}

        .all-option.selected {{
            background: #007bff;
            color: white;
            border-color: #007bff;
        }}

        .mini-grid-canvas {{
            border: 2px solid #333;
            margin: 5px auto;
            background: #f8f9fa;
            display: block;
        }}

        .filter-grid-label {{
            font-size: 0.85rem;
            color: #666;
            margin-top: 5px;
        }}

        .algo-filter-container {{
            border: 1px solid #ddd;
            border-radius: 4px;
            padding: 10px;
            background: white;
            display: grid;
            grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
            gap: 5px;
        }}

        .algo-checkbox-item {{
            display: flex;
            align-items: center;
            padding: 8px 10px;
            margin: 0;
            border: 1px solid #e0e0e0;
            border-radius: 4px;
            background: #f8f9fa;
            transition: all 0.2s;
        }}

        .algo-checkbox-item:hover {{
            background: #e7f3ff;
            border-color: #007bff;
        }}

        .algo-checkbox-item input[type="checkbox"] {{
            margin-right: 8px;
            cursor: pointer;
        }}

        .algo-checkbox-item label {{
            cursor: pointer;
            margin: 0;
            font-size: 0.9rem;
            font-weight: normal;
        }}

        .algo-filter-buttons {{
            display: flex;
            gap: 10px;
            margin-bottom: 10px;
        }}

        .algo-filter-buttons button {{
            padding: 5px 15px;
            border: 1px solid #007bff;
            background: white;
            color: #007bff;
            border-radius: 4px;
            cursor: pointer;
            font-size: 0.85rem;
        }}

        .algo-filter-buttons button:hover {{
            background: #007bff;
            color: white;
        }}
    </style>
</head>
<body>
    <div class="container">
        <!-- Header -->
        <div class="header">
            <h1>Multi-Algorithm Execution Viewer: <span id="algoCount">...</span> algorithms</h1>
            <p style="color: #666; margin: 0;">View and compare execution of all algorithms for a single experiment</p>
        </div>

        <!-- Experiment Selector -->
        <div class="experiment-selector">
            <label for="experimentSelect">Select Experiment</label>
            <select id="experimentSelect" class="form-select">
                <option value="">-- Select an Experiment --</option>
            </select>
            
            <!-- Filter Toggle Links -->
            <div id="filterToggleContainer" style="display: none;">
                <span id="filterToggleLink" class="filter-toggle-link" onclick="toggleFiltersMulti()">🔽 Refine experiment selection</span>
                <span> | </span>
                <span id="algoFilterToggleLink" class="filter-toggle-link" onclick="toggleAlgoFilter()">🔽 Show algorithm filters</span>
            </div>
            
            <!-- Experiment Filter Section -->
            <div id="filterSection" class="filter-section">
                <label>Filter by Initial Positions</label>
                <div id="initialPosGridList" class="filter-grid-list">
                    <!-- Will be populated by JavaScript -->
                </div>
                <label style="margin-top: 15px;">Filter by Grid Config</label>
                <select id="gridConfigFilter" class="form-select">
                    <option value="">-- All Grid Configs --</option>
                </select>
            </div>
            
            <!-- Algorithm Filter Section -->
            <div id="algoFilterSection" class="filter-section">
                <label>Filter by Algorithms</label>
                <div class="algo-filter-buttons">
                    <button onclick="selectAllAlgos()">Select All</button>
                    <button onclick="unselectAllAlgos()">Unselect All</button>
                </div>
                <div id="algoFilterList" class="algo-filter-container">
                    <!-- Will be populated by JavaScript -->
                </div>
            </div>
        </div>

        <!-- Experiment Info -->
        <div id="experimentInfo" class="control-panel" style="display: none;">
            <div id="experimentInfoContent" style="font-family: monospace; font-size: 0.9rem; color: #333; line-height: 1.8;"></div>
        </div>

        <!-- Control Panel -->
        <div id="controlPanel" class="control-panel" style="display: none;">
            <div class="animation-controls">
                <button onclick="playAnimation()">▶ Play</button>
                <button onclick="pauseAnimation()">⏸ Pause</button>
                <button onclick="prevStep()">◀ Prev</button>
                <button onclick="nextStep()">▶ Next</button>
                <button onclick="resetAnimation()">⟲ Reset</button>
                <button class="find-diff" onclick="findNextDifference()">🔍 Find Next Difference</button>
                <span class="step-info" id="stepInfo">Step: 0 / 0</span>
                <label style="margin-left: 10px;">
                    Jump to step: 
                    <input type="number" id="stepInput" min="0" max="0" value="0" style="width: 70px; padding: 5px;" oninput="jumpToStep()" />
                </label>
            </div>
        </div>

        <!-- Grids Container -->
        <div id="gridsContainer" class="grids-container row" style="display: none;">
            <!-- Will be populated by JavaScript -->
        </div>

        <!-- Empty State -->
        <div id="noSelection" class="no-selection">
            <h3>Select an Experiment</h3>
            <p>Choose an experiment from the list above to view execution across all algorithms</p>
        </div>
    </div>

    <script src="https://cdn.jsdelivr.net/npm/bootstrap@5.3.2/dist/js/bootstrap.bundle.min.js"></script>
    <script src="multiview_core_data/common_data.js"></script>
    <script>
        let currentExperimentIndex = -1;
        let currentStep = 0;
        let maxSteps = 0;
        let playInterval = null;
        let parsedExperiments = [];
        let selectedInitialPosFilter = '';
        let selectedGridConfigFilter = '';
        let selectedAlgos = new Set(); // Track selected algorithms

        // Natural sort function for algorithm names
        function naturalSort(a, b) {{
            const ax = [];
            const bx = [];
            
            a.replace(/(\d+)|(\D+)/g, (_, num, str) => {{
                ax.push([num || Infinity, str || '']);
            }});
            b.replace(/(\d+)|(\D+)/g, (_, num, str) => {{
                bx.push([num || Infinity, str || '']);
            }});
            
            while (ax.length && bx.length) {{
                const an = ax.shift();
                const bn = bx.shift();
                const nn = (an[0] - bn[0]) || an[1].localeCompare(bn[1]);
                if (nn) return nn;
            }}
            
            return ax.length - bx.length;
        }}

        // Draw grid on canvas
        function drawGrid(canvas, gridData, stepIndex, algoIndex) {{
            if (!gridData || !gridData.steps) return;
            
            const ctx = canvas.getContext('2d');
            const {{ minX, maxX, minY, maxY, obstacleX, obstacleY, steps }} = gridData;
            
            const isLastStateRepeated = stepIndex >= steps.length;
            const actualStepIndex = isLastStateRepeated ? steps.length - 1 : stepIndex;
            
            const FRAME_SCALE = 25;
            const PADDING = 2;  // Minimal padding - very tight fit
            
            const simConfig = {{
                colors: {{ 
                    L: 'red', R: 'green', F: 'blue', O: 'orange',
                    r: 'red', g: 'green', b: 'blue', o: 'orange',
                    B: 'blue', G: 'green', Y: 'yellow', P: 'purple',
                    y: 'yellow', p: 'purple', W: '#333', w: '#333'
                }},
                boundaryColor: '#666',
                boundaryWidth: 2,
                robotRadius: 8
            }};
            
            const gridWidth = maxX - minX ;
            const gridHeight = maxY - minY ;
            const canvasWidth = gridWidth * FRAME_SCALE + 2 * PADDING;
            const canvasHeight = gridHeight * FRAME_SCALE + 2 * PADDING;
            
            // Set canvas dimensions
            canvas.width = canvasWidth;
            canvas.height = canvasHeight;
            
            function frameWorldToCanvas(x, y) {{
                return {{
                    cx: PADDING + (x - minX) * FRAME_SCALE,
                    cy: canvasHeight - PADDING - (y - minY) * FRAME_SCALE
                }};
            }}
            
            ctx.clearRect(0, 0, canvas.width, canvas.height);
            
            // Draw grid lines
            ctx.strokeStyle = '#e8e8e8';
            ctx.lineWidth = 1;
            ctx.setLineDash([]);
            
            for (let x = minX; x <= maxX; x++) {{
                const {{ cx }} = frameWorldToCanvas(x, minY);
                const {{ cy: cy_top }} = frameWorldToCanvas(x, maxY);
                const {{ cy: cy_bottom }} = frameWorldToCanvas(x, minY);
                ctx.beginPath();
                ctx.moveTo(cx + 0.5, cy_top);
                ctx.lineTo(cx + 0.5, cy_bottom);
                ctx.stroke();
            }}
            
            for (let y = minY; y <= maxY; y++) {{
                const {{ cx: cx_left }} = frameWorldToCanvas(minX, y);
                const {{ cx: cx_right }} = frameWorldToCanvas(maxX, y);
                const {{ cy }} = frameWorldToCanvas(minX, y);
                ctx.beginPath();
                ctx.moveTo(cx_left, cy + 0.5);
                ctx.lineTo(cx_right, cy + 0.5);
                ctx.stroke();
            }}
            
            // Draw boundaries
            ctx.strokeStyle = simConfig.boundaryColor;
            ctx.lineWidth = simConfig.boundaryWidth;
            ctx.setLineDash([5, 3]);
            
            const {{ cx: x0 }} = frameWorldToCanvas(minX, minY);
            const {{ cy: y0 }} = frameWorldToCanvas(minX, minY);
            const {{ cx: x1 }} = frameWorldToCanvas(maxX, maxY);
            const {{ cy: y1 }} = frameWorldToCanvas(maxX, maxY);
            
            ctx.strokeRect(x0, y1, x1 - x0, y0 - y1);
            ctx.setLineDash([]);
            
            // Draw obstacle
            if (obstacleX !== null && obstacleY !== null) {{
                const {{ cx: obsCx, cy: obsCy }} = frameWorldToCanvas(obstacleX, obstacleY);
                ctx.fillStyle = 'orange';
                ctx.beginPath();
                ctx.arc(obsCx, obsCy, simConfig.robotRadius, 0, Math.PI * 2);
                ctx.fill();
                ctx.strokeStyle = '#333';
                ctx.lineWidth = 1.5;
                ctx.stroke();
            }}
            
            // Draw robots
            const currentStep = steps[actualStepIndex];
            if (currentStep) {{
                currentStep.forEach(robot => {{
                    const {{ cx, cy }} = frameWorldToCanvas(robot.x, robot.y);
                    const isInBounds = robot.x >= minX && robot.x <= maxX && 
                                       robot.y >= minY && robot.y <= maxY;
                    
                    ctx.fillStyle = simConfig.colors[robot.c] || 'gray';
                    ctx.beginPath();
                    ctx.arc(cx, cy, simConfig.robotRadius, 0, Math.PI * 2);
                    ctx.fill();
                    
                    ctx.strokeStyle = isInBounds ? '#333' : '#e53935';
                    ctx.lineWidth = isInBounds ? 1.5 : 2;
                    ctx.stroke();
                }});
            }}
            
            // Draw X if beyond available steps
            if (isLastStateRepeated) {{
                ctx.strokeStyle = '#e53935';
                ctx.lineWidth = 4;
                const xSize = 40;
                const centerX = (x0 + x1) / 2;
                const centerY = (y0 + y1) / 2;
                
                ctx.beginPath();
                ctx.moveTo(centerX - xSize/2, centerY - xSize/2);
                ctx.lineTo(centerX + xSize/2, centerY + xSize/2);
                ctx.moveTo(centerX + xSize/2, centerY - xSize/2);
                ctx.lineTo(centerX - xSize/2, centerY + xSize/2);
                ctx.stroke();
                
                ctx.fillStyle = '#e53935';
                ctx.font = 'bold 14px Arial';
                ctx.textAlign = 'center';
                ctx.fillText('Step Not Available', centerX, centerY + xSize/2 + 20);
            }}
        }}

        // Populate experiment dropdown
        function populateExperimentList() {{
            const experimentSelect = document.getElementById('experimentSelect');
            experimentSelect.innerHTML = '<option value="">-- Select an Experiment --</option>';
            
            if (!window.commonData || !window.commonData.experiments) return;
            
            let displayIndex = 0;
            let firstMatchingIndex = -1;
            
            window.commonData.experiments.forEach((exp) => {{
                // Add to dropdown
                const option = document.createElement('option');
                option.value = exp.id;
                option.textContent = `Experiment ${{exp.id}}`;
                experimentSelect.appendChild(option);
            }}); 
            
            // Add change event listener
            experimentSelect.onchange = (e) => {{
                const index = parseInt(e.target.value);
                if (!isNaN(index)) {{
                    selectExperiment(index);
                }} else {{
                    // Hide grids if no selection
                    document.getElementById('noSelection').style.display = 'block';
                    document.getElementById('controlPanel').style.display = 'none';
                    document.getElementById('gridsContainer').style.display = 'none';
                    document.getElementById('experimentInfo').style.display = 'none';
                    document.getElementById('filterToggleContainer').style.display = 'none';
                }}
            }};
        }}

        // Find experiment based on selected filters
        function findMatchingExperiment() {{
            if (!selectedInitialPosFilter || !selectedGridConfigFilter) return;
            
            if (!window.commonData || !window.commonData.experiments) return;
            
            // Find first matching experiment
            const match = window.commonData.experiments.find(exp => 
                exp.initial_positions === selectedInitialPosFilter && 
                exp.grid_config_sig === selectedGridConfigFilter
            );
            
            if (match) {{
                // Select this experiment
                selectExperiment(match.id);
                
                // Sync dropdown
                const experimentSelect = document.getElementById('experimentSelect');
                if (experimentSelect) {{
                    experimentSelect.value = match.id;
                }}
            }}
        }}

        // Toggle experiment filter visibility
        function toggleFiltersMulti() {{
            const filterSection = document.getElementById('filterSection');
            const algoFilterSection = document.getElementById('algoFilterSection');
            const toggleLink = document.getElementById('filterToggleLink');
            
            if (filterSection.classList.contains('visible')) {{
                filterSection.classList.remove('visible');
                toggleLink.textContent = '🔽 Refine experiment selection';
            }} else {{
                filterSection.classList.add('visible');
                toggleLink.textContent = '🔼 Hide experiment selection';
                // Close algorithm filter if open
                if (algoFilterSection.classList.contains('visible')) {{
                    algoFilterSection.classList.remove('visible');
                    document.getElementById('algoFilterToggleLink').textContent = '🔽 Show algorithm filters';
                }}
            }}
        }}

        // Toggle algorithm filter visibility
        function toggleAlgoFilter() {{
            const filterSection = document.getElementById('filterSection');
            const algoFilterSection = document.getElementById('algoFilterSection');
            const toggleLink = document.getElementById('algoFilterToggleLink');
            
            if (algoFilterSection.classList.contains('visible')) {{
                algoFilterSection.classList.remove('visible');
                toggleLink.textContent = '🔽 Show algorithm filters';
            }} else {{
                algoFilterSection.classList.add('visible');
                toggleLink.textContent = '🔼 Hide algorithm filters';
                // Close experiment filter if open
                if (filterSection.classList.contains('visible')) {{
                    filterSection.classList.remove('visible');
                    document.getElementById('filterToggleLink').textContent = '🔽 Refine experiment selection';
                }}
            }}
        }}

        // Populate filters based on common data
        function populateFiltersMulti() {{
            if (!window.commonData || !window.commonData.experiments) return;
            
            // Update algorithm count in header
            if (window.commonData.algorithms) {{
                const algoCount = [...new Set(window.commonData.algorithms.map(a => a.name))].length;
                document.getElementById('algoCount').textContent = algoCount;
            }}
            
            // Populate algorithm filter checkboxes
            const algoFilterList = document.getElementById('algoFilterList');
            if (algoFilterList && window.commonData.algorithms) {{
                algoFilterList.innerHTML = '';
                
                // Get all unique algorithm names and sort them
                const algoNames = [...new Set(window.commonData.algorithms.map(a => a.name))];
                algoNames.sort(naturalSort);
                
                // Initialize selectedAlgos with all algorithms if empty
                if (selectedAlgos.size === 0) {{
                    algoNames.forEach(name => selectedAlgos.add(name));
                }}
                
                algoNames.forEach(algoName => {{
                    const checkboxDiv = document.createElement('div');
                    checkboxDiv.className = 'algo-checkbox-item';
                    
                    const checkbox = document.createElement('input');
                    checkbox.type = 'checkbox';
                    checkbox.id = `algo_${{algoName}}`;
                    checkbox.value = algoName;
                    checkbox.checked = selectedAlgos.has(algoName);
                    checkbox.onchange = (e) => {{
                        if (e.target.checked) {{
                            selectedAlgos.add(algoName);
                        }} else {{
                            selectedAlgos.delete(algoName);
                        }}
                        // Refresh the grids display if an experiment is selected
                        if (currentExperimentIndex !== -1) {{
                            updateGridsDisplay();
                        }}
                    }};
                    
                    const label = document.createElement('label');
                    label.htmlFor = `algo_${{algoName}}`;
                    label.textContent = algoName;
                    
                    checkboxDiv.appendChild(checkbox);
                    checkboxDiv.appendChild(label);
                    algoFilterList.appendChild(checkboxDiv);
                }});
            }}
            
            // Collect unique initial positions and grid configs
            const initialPositionsSet = new Set();
            const gridConfigsSet = new Set();
            
            window.commonData.experiments.forEach(exp => {{
                initialPositionsSet.add(exp.initial_positions);
                gridConfigsSet.add(exp.grid_config_sig);
            }});
            
            // Populate initial positions grid
            const initialPosGridList = document.getElementById('initialPosGridList');
            initialPosGridList.innerHTML = '';
            
            // Add visual grid for each unique initial position
            let posIndex = 0;
            Array.from(initialPositionsSet).sort().forEach(initPos => {{
                const gridOption = document.createElement('div');
                gridOption.className = 'filter-grid-option' + (selectedInitialPosFilter === initPos ? ' selected' : '');
                
                const canvasId = `miniCanvas_${{posIndex}}`;
                gridOption.innerHTML = `
                    <canvas id="${{canvasId}}" class="mini-grid-canvas" width="120" height="120"></canvas>
                    <div class="filter-grid-label">Config ${{posIndex + 1}}</div>
                `;
                gridOption.onclick = () => {{
                    selectedInitialPosFilter = initPos;
                    populateFiltersMulti(); // Refresh UI selection
                    populateExperimentList(); // Filter list
                    findMatchingExperiment(); // Auto-select match
                }};
                initialPosGridList.appendChild(gridOption);
                
                // Draw mini grid
                setTimeout(() => {{
                    const canvas = document.getElementById(canvasId);
                    if (canvas) drawMiniGrid(canvas, initPos);
                }}, 10);
                
                posIndex++;
            }});
            
            // Populate grid config filter
            const gridConfigFilter = document.getElementById('gridConfigFilter');
            gridConfigFilter.innerHTML = '<option value="">-- All Grid Configs --</option>';
            Array.from(gridConfigsSet).sort().forEach(gridConfig => {{
                const option = document.createElement('option');
                option.value = gridConfig;
                option.textContent = gridConfig;
                if (selectedGridConfigFilter === gridConfig) {{
                    option.selected = true;
                }}
                gridConfigFilter.appendChild(option);
            }});
            
            // Add event listener for grid config filter
            gridConfigFilter.onchange = () => {{
                selectedGridConfigFilter = gridConfigFilter.value;
                populateExperimentList();
                findMatchingExperiment(); // Auto-select match
            }};
        }}

        // Draw mini grid for initial position visualization
        function drawMiniGrid(canvas, initPosStr) {{
            const ctx = canvas.getContext('2d');
            const FRAME_SCALE = 20;
            const FRAME_THICKNESS = 3;
            const FRAME_PADDING = 1;
            
            const simConfig = {{
                colors: {{ 
                    L: 'red', R: 'green', F: 'blue', O: 'orange',
                    r: 'red', g: 'green', b: 'blue', o: 'orange',
                    B: 'blue', G: 'green', Y: 'yellow', P: 'purple',
                    y: 'yellow', p: 'purple', W: '#333', w: '#333'
                }},
                robotRadius: 8
            }};
            
            // Parse initial positions
            const positions = [];
            const matches = initPosStr.matchAll(/\(([^,]+),([^,]+),([^)]+)\)/g);
            for (const match of matches) {{
                positions.push({{
                    c: match[1],
                    x: parseInt(match[2]),
                    y: parseInt(match[3])
                }});
            }}
            
            const obstacleX = 0;
            const obstacleY = 0;
            
            ctx.clearRect(0, 0, canvas.width, canvas.height);
            
            // Find bounds
            let minX = 0, maxX = 0, minY = 0, maxY = 0;
            positions.forEach(pos => {{
                minX = Math.min(minX, pos.x);
                maxX = Math.max(maxX, pos.x);
                minY = Math.min(minY, pos.y);
                maxY = Math.max(maxY, pos.y);
            }});
            
            // Extend bounds
            const frameMinX = minX - FRAME_PADDING;
            const frameMaxX = maxX + FRAME_PADDING;
            const frameMinY = minY - FRAME_PADDING;
            const frameMaxY = maxY + FRAME_PADDING;
            
            const frameGridWidth = frameMaxX - frameMinX;
            const frameGridHeight = frameMaxY - frameMinY;
            const framePixelWidth = frameGridWidth * FRAME_SCALE;
            const framePixelHeight = frameGridHeight * FRAME_SCALE;
            
            const frameX = (canvas.width - framePixelWidth) / 2;
            const frameY = (canvas.height - framePixelHeight) / 2;
            
            function toCanvas(x, y) {{
                return {{
                    cx: frameX + (x - frameMinX) * FRAME_SCALE,
                    cy: frameY + framePixelHeight - (y - frameMinY) * FRAME_SCALE
                }};
            }}
            
            // Draw grid lines
            ctx.strokeStyle = '#e8e8e8';
            ctx.lineWidth = 1;
            ctx.setLineDash([]);
            
            for (let x = frameMinX; x <= frameMaxX; x++) {{
                const {{ cx }} = toCanvas(x, frameMinY);
                const {{ cy: cy_top }} = toCanvas(x, frameMaxY);
                const {{ cy: cy_bottom }} = toCanvas(x, frameMinY);
                ctx.beginPath();
                ctx.moveTo(cx + 0.5, cy_top);
                ctx.lineTo(cx + 0.5, cy_bottom);
                ctx.stroke();
            }}
            
            for (let y = frameMinY; y <= frameMaxY; y++) {{
                const {{ cx: cx_left }} = toCanvas(frameMinX, y);
                const {{ cx: cx_right }} = toCanvas(frameMaxX, y);
                const {{ cy }} = toCanvas(frameMinX, y);
                ctx.beginPath();
                ctx.moveTo(cx_left, cy + 0.5);
                ctx.lineTo(cx_right, cy + 0.5);
                ctx.stroke();
            }}
            
            // Draw frame border
            ctx.strokeStyle = '#333';
            ctx.lineWidth = FRAME_THICKNESS;
            ctx.setLineDash([]);
            ctx.strokeRect(frameX, frameY, framePixelWidth, framePixelHeight);
            
            // Draw obstacle
            const {{ cx: obsCx, cy: obsCy }} = toCanvas(obstacleX, obstacleY);
            ctx.fillStyle = 'orange';
            ctx.beginPath();
            ctx.arc(obsCx, obsCy, simConfig.robotRadius, 0, Math.PI * 2);
            ctx.fill();
            ctx.strokeStyle = '#333';
            ctx.lineWidth = 1.5;
            ctx.stroke();
            
            // Draw robots
            positions.forEach(robot => {{
                const {{ cx, cy }} = toCanvas(robot.x, robot.y);
                ctx.fillStyle = simConfig.colors[robot.c] || 'gray';
                ctx.beginPath();
                ctx.arc(cx, cy, simConfig.robotRadius, 0, Math.PI * 2);
                ctx.fill();
                ctx.strokeStyle = '#333';
                ctx.lineWidth = 1.5;
                ctx.stroke();
            }});
        }}

        // Select experiment
        function selectExperiment(index) {{
            currentExperimentIndex = index;
            currentStep = 0;
            
            // Sync filters with this experiment
            if (window.commonData && window.commonData.experiments) {{
                const exp = window.commonData.experiments.find(e => e.id === index);
                if (exp) {{
                    selectedInitialPosFilter = exp.initial_positions;
                    selectedGridConfigFilter = exp.grid_config_sig;
                    
                    // Update UI
                    populateFiltersMulti();
                    const gridConfigSelect = document.getElementById('gridConfigFilter');
                    if (gridConfigSelect) {{
                        gridConfigSelect.value = selectedGridConfigFilter;
                    }}
                }}
            }}
            
            // Dynamic load of experiment data
            const scriptId = 'dynamic-exp-script';
            const oldScript = document.getElementById(scriptId);
            if (oldScript) oldScript.remove();
            
            const script = document.createElement('script');
            script.id = scriptId;
            script.src = `multiview_core_data/experiment_${{index}}.js`;
            script.onload = () => {{
                const expData = window[`experimentData_${{index}}`];
                loadExperimentData(expData);
            }};
            document.body.appendChild(script);
        }}

        function loadExperimentData(expData) {{
            if (!expData) return;
            
            const commonExp = window.commonData.experiments[currentExperimentIndex];
            
            // Parse common grid info once
            const gridPart = commonExp.grid_info.replace('GRID:', '');
            const gridValues = gridPart.split(',').map(Number);
            const [minX, maxX, minY, maxY] = gridValues;
            const columns = gridValues.length > 4 ? gridValues[4] : null;
            const rows = gridValues.length > 5 ? gridValues[5] : null;
            const obstacleX = gridValues.length > 6 ? gridValues[6] : null;
            const obstacleY = gridValues.length > 7 ? gridValues[7] : null;
            
            parsedExperiments = expData.map(algoExec => {{
                const historyPart = algoExec.history;
                const steps = historyPart.split(';').map(step => {{
                    return step.split(',').reduce((acc, part, idx, arr) => {{
                        if (idx % 3 === 0) {{
                            const c = part.replace('(', '');
                            const x = parseInt(arr[idx + 1]);
                            const y = parseInt(arr[idx + 2].replace(')', ''));
                            acc.push({{ c, x, y }});
                        }}
                        return acc;
                    }}, []);
                }});
                
                return {{
                    name: algoExec.algo_name,
                    status: algoExec.status,
                    cycleLen: algoExec.cycle_len || 0,
                    data: {{
                        minX, maxX, minY, maxY, columns, rows, obstacleX, obstacleY,
                        steps,
                        initialPositions: commonExp.initial_positions,
                        gridConfigSig: commonExp.grid_config_sig
                    }}
                }};
            }});
            
            // Sort experiments by name using natural sort
            parsedExperiments.sort((a, b) => naturalSort(a.name, b.name));
            
            // Find max steps
            maxSteps = Math.max(...parsedExperiments.map(exp => 
                exp.data ? exp.data.steps.length : 0
            ));
            
            // Update experiment info display
            const firstExp = parsedExperiments[0].data;
            if (firstExp) {{
                const stepsInfo = parsedExperiments.map(exp => exp.data ? exp.data.steps.length - 1 : 0).join(' vs ');
                document.getElementById('experimentInfoContent').innerHTML = `
                    <strong>Grid:</strong> x[${{firstExp.minX}}, ${{firstExp.maxX}}] y[${{firstExp.minY}}, ${{firstExp.maxY}}] | 
                    <strong>Columns:</strong> ${{firstExp.columns !== null ? firstExp.columns : '?'}}, 
                    <strong>Rows:</strong> ${{firstExp.rows !== null ? firstExp.rows : '?'}} | 
                    <strong>Obstacle:</strong> (${{firstExp.obstacleX !== null ? firstExp.obstacleX : '?'}}, ${{firstExp.obstacleY !== null ? firstExp.obstacleY : '?'}}) | 
                    <strong>Steps:</strong> ${{stepsInfo}}<br>
                    <strong>Initial Position:</strong> ${{firstExp.initialPositions}}
                `;
                document.getElementById('experimentInfo').style.display = 'block';
            }}
            
            // Update UI
            document.getElementById('noSelection').style.display = 'none';
            document.getElementById('controlPanel').style.display = 'block';
            document.getElementById('gridsContainer').style.display = 'flex';
            document.getElementById('filterToggleContainer').style.display = 'block';
            
            // Create grid containers
            updateGridsDisplay();
            
            // Update step input
            document.getElementById('stepInput').max = maxSteps - 1;
            document.getElementById('stepInput').value = 0;
            
            // Draw initial state
            drawAllGrids();
        }}

        // Update grids display based on selected algorithms filter
        function updateGridsDisplay() {{
            const gridsContainer = document.getElementById('gridsContainer');
            gridsContainer.innerHTML = '';
            
            const colClass = 'col-xl-4 col-lg-6 col-md-6 col-sm-12';
            
            parsedExperiments.forEach((exp, idx) => {{
                // Only display if algorithm is selected
                if (!selectedAlgos.has(exp.name)) return;
                
                const algoStatus = exp.status;
                
                const colDiv = document.createElement('div');
                colDiv.className = colClass;
                
                const gridDiv = document.createElement('div');
                gridDiv.className = 'algorithm-grid';
                gridDiv.id = `algoGrid_${{idx}}`;
                gridDiv.innerHTML = `
                    <h3>${{exp.name}} <span style="font-size: 0.85rem; color: #666;">[${{algoStatus}}]</span></h3>
                    <canvas id="canvas_${{idx}}" class="grid-canvas"></canvas>
                    <div style="font-size: 0.9rem; color: #333; margin-top: 8px; display: flex; justify-content: space-between; padding: 0 10px;">
                        <span><strong>Step:</strong> <span id="absoluteStep_${{idx}}">0</span> / ${{exp.data ? exp.data.steps.length - 1 : 0}}</span>
                        <span><strong>Cycle Step:</strong> <span id="cycleStep_${{idx}}">-</span> / ${{exp.cycleLen || 0}}</span>
                    </div>
                `;
                
                colDiv.appendChild(gridDiv);
                gridsContainer.appendChild(colDiv);
            }});
            
            // Redraw all grids after updating display
            drawAllGrids();
        }}

        // Select all algorithms
        function selectAllAlgos() {{
            if (!window.commonData || !window.commonData.algorithms) return;
            
            selectedAlgos.clear();
            window.commonData.algorithms.forEach(algo => selectedAlgos.add(algo.name));
            
            // Update all checkboxes
            document.querySelectorAll('.algo-checkbox-item input[type=\"checkbox\"]').forEach(checkbox => {{
                checkbox.checked = true;
            }});
            
            // Refresh display
            if (currentExperimentIndex !== -1) {{
                updateGridsDisplay();
            }}
        }}

        // Unselect all algorithms
        function unselectAllAlgos() {{
            selectedAlgos.clear();
            
            // Update all checkboxes
            document.querySelectorAll('.algo-checkbox-item input[type=\"checkbox\"]').forEach(checkbox => {{
                checkbox.checked = false;
            }});
            
            // Refresh display
            if (currentExperimentIndex !== -1) {{
                updateGridsDisplay();
            }}
        }}

        // Draw all grids at current step
        function drawAllGrids() {{
            // Use the first algorithm's grid bounds as reference for all grids
            const referenceGridData = parsedExperiments[0] && parsedExperiments[0].data ? 
                                      parsedExperiments[0].data : null;
            
            parsedExperiments.forEach((exp, idx) => {{
                const canvas = document.getElementById(`canvas_${{idx}}`);
                if (canvas && exp.data) {{
                    drawGrid(canvas, exp.data, currentStep, idx);
                }}
                
                // Update absolute step counter
                const absoluteStepElem = document.getElementById(`absoluteStep_${{idx}}`);
                if (absoluteStepElem) {{
                    absoluteStepElem.textContent = currentStep;
                }}
                
                // Update cycle step counter for this algorithm
                const cycleStepElem = document.getElementById(`cycleStep_${{idx}}`);
                if (cycleStepElem && exp.cycleLen) {{
                    const totalSteps = exp.data ? exp.data.steps.length : 0;
                    const cycleStartStep = totalSteps - exp.cycleLen;
                    const cycleRelativeStep = currentStep - cycleStartStep + 1;
                    cycleStepElem.textContent = cycleRelativeStep;
                    
                    // Color code: negative (before cycle) in blue, positive (in cycle) in green
                    if (cycleRelativeStep < 0) {{
                        cycleStepElem.style.color = '#0066cc';
                    }} else {{
                        cycleStepElem.style.color = '#28a745';
                        cycleStepElem.style.fontWeight = 'bold';
                    }}
                }} else if (cycleStepElem) {{
                    cycleStepElem.textContent = 'N/A';
                    cycleStepElem.style.color = '#999';
                }}
            }});
            
            // Update step info
            document.getElementById('stepInfo').textContent = `Step: ${{currentStep}} / ${{maxSteps - 1}}`;
            document.getElementById('stepInput').value = currentStep;
            
            // Highlight different grids
            highlightDifferences();
        }}

        // Check and highlight differences
        function highlightDifferences() {{
            if (parsedExperiments.length < 2) return;
            
            // Get current step for all algorithms
            const currentSteps = parsedExperiments.map(exp => {{
                if (!exp.data || currentStep >= exp.data.steps.length) {{
                    return exp.data && exp.data.steps.length > 0 ? 
                           exp.data.steps[exp.data.steps.length - 1] : [];
                }}
                return exp.data.steps[currentStep];
            }});
            
            // Compare each algorithm with others
            const isDifferent = new Array(parsedExperiments.length).fill(false);
            
            for (let i = 0; i < currentSteps.length; i++) {{
                for (let j = i + 1; j < currentSteps.length; j++) {{
                    if (!stepsEqual(currentSteps[i], currentSteps[j])) {{
                        isDifferent[i] = true;
                        isDifferent[j] = true;
                    }}
                }}
            }}
            
            // Apply highlighting
            parsedExperiments.forEach((exp, idx) => {{
                const gridDiv = document.getElementById(`algoGrid_${{idx}}`);
                if (gridDiv) {{
                    gridDiv.classList.toggle('different', isDifferent[idx]);
                }}
            }});
        }}

        // Compare two steps
        function stepsEqual(step1, step2) {{
            if (!step1 || !step2) return false;
            if (step1.length !== step2.length) return false;
            
            const sorted1 = step1.slice().sort((a, b) => {{
                if (a.x !== b.x) return a.x - b.x;
                if (a.y !== b.y) return a.y - b.y;
                return a.c.localeCompare(b.c);
            }});
            const sorted2 = step2.slice().sort((a, b) => {{
                if (a.x !== b.x) return a.x - b.x;
                if (a.y !== b.y) return a.y - b.y;
                return a.c.localeCompare(b.c);
            }});
            
            for (let i = 0; i < sorted1.length; i++) {{
                if (sorted1[i].x !== sorted2[i].x || 
                    sorted1[i].y !== sorted2[i].y || 
                    sorted1[i].c !== sorted2[i].c) {{
                    return false;
                }}
            }}
            
            return true;
        }}

        // Animation controls
        function playAnimation() {{
            if (playInterval) return;
            playInterval = setInterval(() => {{
                currentStep = (currentStep + 1) % maxSteps;
                drawAllGrids();
            }}, 500);
        }}

        function pauseAnimation() {{
            if (playInterval) {{
                clearInterval(playInterval);
                playInterval = null;
            }}
        }}

        function nextStep() {{
            pauseAnimation();
            currentStep = Math.min(currentStep + 1, maxSteps - 1);
            drawAllGrids();
        }}

        function prevStep() {{
            pauseAnimation();
            currentStep = Math.max(currentStep - 1, 0);
            drawAllGrids();
        }}

        function resetAnimation() {{
            pauseAnimation();
            currentStep = 0;
            drawAllGrids();
        }}

        function jumpToStep() {{
            pauseAnimation();
            const targetStep = parseInt(document.getElementById('stepInput').value);
            if (!isNaN(targetStep) && targetStep >= 0 && targetStep < maxSteps) {{
                currentStep = targetStep;
                drawAllGrids();
            }}
        }}

        function findNextDifference() {{
            pauseAnimation();
            
            const startStep = currentStep + 1;
            
            // Search from current step to end
            for (let step = startStep; step < maxSteps; step++) {{
                const stepsAtThisPoint = parsedExperiments.map(exp => {{
                    if (!exp.data || step >= exp.data.steps.length) {{
                        return exp.data && exp.data.steps.length > 0 ? 
                               exp.data.steps[exp.data.steps.length - 1] : [];
                    }}
                    return exp.data.steps[step];
                }});
                
                // Check if any two are different
                let foundDiff = false;
                for (let i = 0; i < stepsAtThisPoint.length && !foundDiff; i++) {{
                    for (let j = i + 1; j < stepsAtThisPoint.length; j++) {{
                        if (!stepsEqual(stepsAtThisPoint[i], stepsAtThisPoint[j])) {{
                            currentStep = step;
                            drawAllGrids();
                            return;
                        }}
                    }}
                }}
            }}
            
            // Wrap around to beginning
            for (let step = 0; step < startStep - 1; step++) {{
                const stepsAtThisPoint = parsedExperiments.map(exp => {{
                    if (!exp.data || step >= exp.data.steps.length) {{
                        return exp.data && exp.data.steps.length > 0 ? 
                               exp.data.steps[exp.data.steps.length - 1] : [];
                    }}
                    return exp.data.steps[step];
                }});
                
                let foundDiff = false;
                for (let i = 0; i < stepsAtThisPoint.length && !foundDiff; i++) {{
                    for (let j = i + 1; j < stepsAtThisPoint.length; j++) {{
                        if (!stepsEqual(stepsAtThisPoint[i], stepsAtThisPoint[j])) {{
                            currentStep = step;
                            drawAllGrids();
                            return;
                        }}
                    }}
                }}
            }}
            
            alert('No differences found - all algorithms have identical execution');
        }}

        // Keyboard navigation
        document.addEventListener('keydown', function(event) {{
            // Only capture if an experiment is selected
            if (currentExperimentIndex !== -1) {{
                if (event.key === 'ArrowRight') {{
                    event.preventDefault();
                    nextStep();
                }} else if (event.key === 'ArrowLeft') {{
                    event.preventDefault();
                    prevStep();
                }} else if (event.key === 'Enter') {{
                    event.preventDefault();
                    findNextDifference();
                }}
            }}
        }});

        // Parse URL parameters
        function getUrlParams() {{
            const params = new URLSearchParams(window.location.search);
            return {{
                experiment: params.get('experiment'),
                algorithms: params.get('algorithms')
            }};
        }}

        // Apply URL parameters
        function applyUrlParams() {{
            const params = getUrlParams();
            
            // Apply algorithm filter if specified
            if (params.algorithms) {{
                const algoIndices = params.algorithms.split(',').map(s => parseInt(s.trim()));
                
                // Clear current selection
                selectedAlgos.clear();
                
                // Get sorted algorithm names
                if (window.commonData && window.commonData.algorithms) {{
                    const algoNames = [...new Set(window.commonData.algorithms.map(a => a.name))];
                    algoNames.sort(naturalSort);
                    
                    // Select only specified algorithms by index
                    algoIndices.forEach(idx => {{
                        if (idx >= 0 && idx < algoNames.length) {{
                            selectedAlgos.add(algoNames[idx]);
                        }}
                    }});
                }}
                
                // Update checkboxes
                document.querySelectorAll('.algo-checkbox-item input[type="checkbox"]').forEach(checkbox => {{
                    checkbox.checked = selectedAlgos.has(checkbox.value);
                }});
            }}
            
            // Update page title if parameters are present
            if (params.experiment !== null || params.algorithms !== null) {{
                let titleParts = [];
                if (params.algorithms) {{
                    titleParts.push(`Algos ${{params.algorithms}}`);
                }}
                if (params.experiment !== null) {{
                    titleParts.push(`Exp. ${{params.experiment}}`);
                }}
                if (titleParts.length > 0) {{
                    document.title = `Animation (${{titleParts.join(' & ')}}) — Execution ${{params.experiment || '0'}} viewer`;
                }}
            }}
            
            // Select experiment if specified, otherwise default to 0
            const expIndex = params.experiment !== null ? parseInt(params.experiment) : 0;
            if (window.commonData.experiments && window.commonData.experiments.length > 0) {{
                if (expIndex >= 0 && expIndex < window.commonData.experiments.length) {{
                    selectExperiment(expIndex);
                    document.getElementById('experimentSelect').value = expIndex;
                }} else {{
                    selectExperiment(0);
                    document.getElementById('experimentSelect').value = 0;
                }}
            }}
        }}

        // Initialize
        if (window.commonData) {{
            populateFiltersMulti();
            populateExperimentList();
            applyUrlParams();
        }} else {{
            // Wait for common data to load if not ready
            window.onload = () => {{
                populateFiltersMulti();
                populateExperimentList();
                applyUrlParams();
            }};
        }}
        

    </script>
</body>
</html>"#
    );

    // Write HTML to file
    fs::write(
        format!("{}/{}viewer.html", output_path, root_name),
        html,
    )
    .expect("Failed to write algorithms viewer HTML file");
    println!(
        "✅ Algorithms viewer HTML generated at: {}",
        format!("{}/{}viewer.html", output_path, root_name)
    );
}
