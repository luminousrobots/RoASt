use rayon::vec;
use serde::{Deserialize, Serialize};

use crate::{
    methodology::globals::{
        get_number_of_colors, get_number_of_robots, get_rules, get_views, get_visibility,
    },
    modules::{
        draft_rules::{self, DraftRule},
        final_rule::FinalRule,
        parallel_rules::{
            self, extract_ending_positions, extract_starting_positions, ParallelRules,
        },
    },
};

pub type Position = (char, i16, i16);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParallelRuleInfo {
    pub id: usize,
    pub rules: Vec<(DraftRule, FinalRule)>,
    pub movable_idle_robots: Vec<Position>,
    pub fixed_idle_robots: Vec<Position>,
    pub active_color_count: usize,
    pub active_movement_count: usize,
    pub starting_positions: Vec<Position>,
    pub ending_positions: Vec<Position>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParallelRulesCollection {
    pub parallel_rules: Vec<ParallelRuleInfo>,
    pub total_count: usize,
    pub visibility_range: i16,
    pub robots_number: usize,
    pub colors_number: usize,
}
pub fn draft_rules_to_final_rules(draft_rules: &[DraftRule]) -> Vec<(DraftRule, FinalRule)> {
    let mut final_rules: Vec<(DraftRule, FinalRule)> = Vec::new();
    for draft_rule in draft_rules {
        let (rule_id, _, _, _, _) = draft_rule;
        let rule = &get_rules()[*rule_id];
        final_rules.push((
            draft_rule.clone(),
            FinalRule {
                view: get_views()[rule.view_id].clone(),
                direction: rule.direction,
                color: rule.color,
            },
        ));
    }
    final_rules
}

pub fn parallel_rules_to_parallel_rules_infos(
    list_of_parallel_rules: &[ParallelRules],
) -> ParallelRulesCollection {
    let list_of_parallel_rules_info = list_of_parallel_rules
        .iter()
        .enumerate()
        .map(|(index, parallel_rules)| {
            let (starting, _) =
                extract_starting_positions(&parallel_rules, &get_views(), &get_rules());
            let (ending, _) = extract_ending_positions(&parallel_rules, &get_rules());
            ParallelRuleInfo {
                id: index,
                rules: draft_rules_to_final_rules(&parallel_rules.rules),
                movable_idle_robots: parallel_rules.movable_idle_robots.clone(),
                fixed_idle_robots: parallel_rules.fixed_idle_robots.clone(),
                active_color_count: parallel_rules.active_color_count,
                active_movement_count: parallel_rules.active_movement_count,
                starting_positions: starting.clone(),
                ending_positions: ending.clone(),
            }
        })
        .collect();
    ParallelRulesCollection {
        parallel_rules: list_of_parallel_rules_info,
        total_count: list_of_parallel_rules.len(),
        visibility_range: get_visibility().clone(),
        robots_number: *get_number_of_robots(),
        colors_number: *get_number_of_colors(),
    }
}

pub fn generate_parallel_rules_html(
    collection: &ParallelRulesCollection,
    output_path: &str,
    number_of_robots: usize,
) {
    use std::fs;

    let json_data = serde_json::to_string(collection).expect("Failed to serialize");

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Parallel Rules Viewer</title>
    <link href="https://cdn.jsdelivr.net/npm/bootstrap@5.3.2/dist/css/bootstrap.min.css" rel="stylesheet">
    <style>
        * {{
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }}
        
        body {{
            background: #fafafa;
            font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', 'Helvetica Neue', Arial, sans-serif;
            padding: 16px;
            color: #1a1a1a;
        }}
        
        #rulesContainer {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
            gap: 16px;
            margin: 16px 0;
        }}
        
        .rule-card {{
            background: white;
            border: 1px solid #e0e0e0;
            border-radius: 4px;
            padding: 12px;
            box-shadow: 0 1px 3px rgba(0,0,0,0.05);
            transition: box-shadow 0.2s;
        }}
        
        .rule-card:hover {{
            box-shadow: 0 2px 8px rgba(0,0,0,0.1);
        }}
        
        .rule-header {{
            background: #2c3e50;
            color: white;
            padding: 8px 12px;
            margin: -12px -12px 12px -12px;
            border-radius: 4px 4px 0 0;
            font-weight: 600;
            font-size: 13px;
        }}
        
        .rule-section {{
            margin: 8px 0;
            padding: 0;
            background: transparent;
            font-size: 12px;
            line-height: 1.6;
        }}
        
        .rule-section strong {{
            color: #555;
            font-weight: 600;
        }}
        
        .rules-list {{
            display: flex;
            gap: 8px;
            flex-wrap: wrap;
            margin-top: 8px;
        }}
        
        .rule-detail-card {{
            background: #f8f9fa;
            border: 1px solid #dee2e6;
            border-radius: 4px;
            padding: 8px;
            flex: 0 0 auto;
        }}
        
        .rule-detail-card h6 {{
            color: #495057;
            margin-bottom: 6px;
            font-size: 11px;
            font-weight: 600;
            text-transform: uppercase;
            letter-spacing: 0.5px;
        }}
        
        .view-canvas {{
            border: 1px solid #e0e0e0;
            border-radius: 2px;
            margin: 6px 0;
            background: white;
            display: block;
        }}
        
        .rule-section canvas.view-canvas {{
            max-width: 100%;
            height: auto;
            display: block;
        }}
        
        .pagination-container {{
            position: sticky;
            top: 0;
            background: white;
            padding: 12px 16px;
            border-bottom: 1px solid #e0e0e0;
            z-index: 1000;
            margin: -16px -16px 16px -16px;
        }}
        
        .pagination-controls {{
            display: flex;
            justify-content: center;
            align-items: center;
            gap: 8px;
            flex-wrap: wrap;
        }}
        
        .pagination-controls button {{
            padding: 6px 12px;
            border: 1px solid #d0d0d0;
            background: white;
            color: #333;
            border-radius: 3px;
            cursor: pointer;
            font-size: 13px;
            font-weight: 500;
            transition: all 0.2s;
        }}
        
        .pagination-controls button:hover:not(:disabled) {{
            background: #f5f5f5;
            border-color: #999;
        }}
        
        .pagination-controls button:disabled {{
            opacity: 0.4;
            cursor: not-allowed;
        }}
        
        .page-info {{
            font-weight: 600;
            color: #333;
            font-size: 13px;
            padding: 0 12px;
        }}
        
        .alert {{
            padding: 12px 16px;
            border-radius: 4px;
            margin-bottom: 16px;
            font-size: 13px;
        }}
        
        .alert-info {{
            background: #e3f2fd;
            border: 1px solid #90caf9;
            color: #1565c0;
        }}
        
        h1 {{
            font-size: 24px;
            font-weight: 600;
            color: #1a1a1a;
            margin-bottom: 16px;
        }}
        
        .filter-toggle {{
            background: white;
            border: 1px solid #e0e0e0;
            border-radius: 4px;
            padding: 8px 12px;
            margin-bottom: 16px;
            cursor: pointer;
            user-select: none;
            display: flex;
            align-items: center;
            gap: 8px;
            font-size: 13px;
            font-weight: 500;
            color: #555;
            transition: background 0.2s;
        }}
        
        .filter-toggle:hover {{
            background: #f8f9fa;
        }}
        
        .filter-panel {{
            background: white;
            border: 1px solid #e0e0e0;
            border-radius: 4px;
            padding: 16px;
            margin-bottom: 16px;
            display: none;
        }}
        
        .filter-panel.active {{
            display: block;
        }}
        
        .filter-row {{
            display: flex;
            gap: 12px;
            margin-bottom: 12px;
            flex-wrap: wrap;
            align-items: flex-end;
        }}
        
        .filter-group {{
            flex: 1;
            min-width: 200px;
        }}
        
        .filter-group label {{
            display: block;
            font-size: 12px;
            font-weight: 600;
            color: #555;
            margin-bottom: 4px;
        }}
        
        .filter-group input,
        .filter-group select {{
            width: 100%;
            padding: 6px 10px;
            border: 1px solid #d0d0d0;
            border-radius: 3px;
            font-size: 13px;
            font-family: inherit;
        }}
        
        .filter-group input:focus,
        .filter-group select:focus {{
            outline: none;
            border-color: #3498db;
        }}
        
        .filter-actions {{
            display: flex;
            gap: 8px;
        }}
        
        .filter-actions button {{
            padding: 6px 16px;
            border: 1px solid #d0d0d0;
            background: white;
            color: #333;
            border-radius: 3px;
            cursor: pointer;
            font-size: 13px;
            font-weight: 500;
            transition: all 0.2s;
        }}
        
        .filter-actions button:hover {{
            background: #f5f5f5;
            border-color: #999;
        }}
        
        .filter-actions button.apply {{
            background: #3498db;
            color: white;
            border-color: #3498db;
        }}
        
        .filter-actions button.apply:hover {{
            background: #2980b9;
            border-color: #2980b9;
        }}
        
        .page-jump {{
            display: flex;
            align-items: center;
            gap: 8px;
        }}
        
        .page-jump input {{
            width: 70px;
            padding: 6px 8px;
            border: 1px solid #d0d0d0;
            border-radius: 3px;
            font-size: 13px;
            text-align: center;
        }}
        
        .page-jump button {{
            padding: 6px 12px;
            border: 1px solid #d0d0d0;
            background: white;
            color: #333;
            border-radius: 3px;
            cursor: pointer;
            font-size: 13px;
            font-weight: 500;
        }}
    </style>
</head>
<body>
    <div class="container-fluid">
        <h1>Parallel Rules Viewer</h1>
        <div class="alert alert-info">
            <strong>Total: <span id="totalCount"></span></strong> | 
            <strong>Visibility: <span id="visibility"></span></strong> | 
            <strong>Robots: <span id="robotsNumber"></span></strong> | 
            <strong>Colors: <span id="colorsNumber"></span></strong>
        </div>
        
        <div class="filter-toggle" onclick="toggleFilters()">
            <span id="filterToggleIcon">▶</span>
            <span id="filterToggleText">Show Filters</span>
        </div>
        
        <div class="filter-panel" id="filterPanel">
            <div style="margin-bottom: 12px;">
                <label style="display: block; margin-bottom: 6px; font-weight: 600; color: #555; font-size: 12px;">Filter by:</label>
                <div style="display: flex; gap: 16px; flex-wrap: wrap;">
                    <label style="font-weight: normal; cursor: pointer; font-size: 13px;">
                        <input type="radio" name="filterType" value="parallelRule" checked onchange="updateFilterInput()"> Parallel Rule Number
                    </label>
                    <label style="font-weight: normal; cursor: pointer; font-size: 13px;">
                        <input type="radio" name="filterType" value="ruleId" onchange="updateFilterInput()"> Rule ID
                    </label>
                    <label style="font-weight: normal; cursor: pointer; font-size: 13px;">
                        <input type="radio" name="filterType" value="ruleCount" onchange="updateFilterInput()"> Number of Rules
                    </label>
                </div>
            </div>
            <div class="filter-row">
                <div class="filter-group">
                    <label id="filterInputLabel">Parallel Rule Number:</label>
                    <input type="number" id="filterNumberInput" placeholder="e.g., 42" min="0" style="display: block;" oninput="applyFilters()">
                    <select id="filterSelectInput" style="display: none;" onchange="applyFilters()">
                        <option value="">All</option>
                        <option value="1">1 rule</option>
                        <option value="2">2 rules</option>
                        <option value="3">3 rules</option>
                        <option value="4">4 rules</option>
                        <option value="5">5 rules</option>
                        <option value="6+">6+ rules</option>
                    </select>
                </div>
            </div>
            <div class="filter-actions">
                <button onclick="clearFilters()">Clear Filter</button>
            </div>
        </div>
        
        <div class="pagination-container">
            <div class="pagination-controls">
                <button id="firstBtn" onclick="goToPage(1)">⏮️ First</button>
                <button id="prevBtn" onclick="goToPage(currentPage - 1)">⬅️ Previous</button>
                <span class="page-info" id="pageInfo"></span>
                <button id="nextBtn" onclick="goToPage(currentPage + 1)">Next ➡️</button>
                <button id="lastBtn" onclick="goToPage(totalPages)">Last ⏭️</button>
                <div class="page-jump">
                    <span style="font-size: 13px;">Go to:</span>
                    <input type="number" id="pageJumpInput" min="1" placeholder="Page" onkeypress="if(event.key==&quot;Enter&quot;) jumpToPage()">
                    <button onclick="jumpToPage()">Go</button>
                </div>
            </div>
        </div>
        
        <div id="rulesContainer"></div>
        
        <div class="pagination-container" style="position: relative;">
            <div class="pagination-controls">
                <button onclick="goToPage(1)">⏮️ First</button>
                <button onclick="goToPage(currentPage - 1)">⬅️ Previous</button>
                <span class="page-info" id="pageInfo2"></span>
                <button onclick="goToPage(currentPage + 1)">Next ➡️</button>
                <button onclick="goToPage(totalPages)">Last ⏭️</button>
                <div class="page-jump">
                    <span style="font-size: 13px;">Go to:</span>
                    <input type="number" id="pageJumpInput2" min="1" placeholder="Page" onkeypress="if(event.key==&quot;Enter&quot;) jumpToPage2()">
                    <button onclick="jumpToPage2()">Go</button>
                </div>
            </div>
        </div>
    </div>

    <script>
        const data = {json_data};
        const numberOfRobots = {number_of_robots};
        
        console.log('Data loaded:', data);
        console.log('Total rules:', data.parallel_rules.length);
        
        document.getElementById('totalCount').textContent = data.total_count;
        document.getElementById('visibility').textContent = data.visibility_range;
        document.getElementById('robotsNumber').textContent = data.robots_number;
        document.getElementById('colorsNumber').textContent = data.colors_number;
        
        // Filtering
        let filteredData = data.parallel_rules;
        let activeFilters = {{}};
        
        function toggleFilters() {{
            const panel = document.getElementById('filterPanel');
            const icon = document.getElementById('filterToggleIcon');
            const text = document.getElementById('filterToggleText');
            
            panel.classList.toggle('active');
            
            if (panel.classList.contains('active')) {{
                icon.textContent = '▼';
                text.textContent = 'Hide Filters';
            }} else {{
                icon.textContent = '▶';
                text.textContent = 'Show Filters';
            }}
        }}
        
        function updateFilterInput() {{
            const filterType = document.querySelector('input[name="filterType"]:checked').value;
            const numberInput = document.getElementById('filterNumberInput');
            const selectInput = document.getElementById('filterSelectInput');
            const label = document.getElementById('filterInputLabel');
            
            // Reset both inputs
            numberInput.value = '';
            selectInput.value = '';
            
            if (filterType === 'parallelRule') {{
                label.textContent = 'Parallel Rule Number:';
                numberInput.style.display = 'block';
                selectInput.style.display = 'none';
                numberInput.placeholder = 'e.g., 42';
            }} else if (filterType === 'ruleId') {{
                label.textContent = 'Rule ID:';
                numberInput.style.display = 'block';
                selectInput.style.display = 'none';
                numberInput.placeholder = 'e.g., 15';
            }} else if (filterType === 'ruleCount') {{
                label.textContent = 'Number of Rules:';
                numberInput.style.display = 'none';
                selectInput.style.display = 'block';
            }}
        }}
        
        function applyFilters() {{
            const filterType = document.querySelector('input[name="filterType"]:checked').value;
            const numberValue = document.getElementById('filterNumberInput').value;
            const selectValue = document.getElementById('filterSelectInput').value;
            
            filteredData = data.parallel_rules.filter(rule => {{
                if (filterType === 'parallelRule' && numberValue) {{
                    // Filter by parallel rule number
                    return rule.id === parseInt(numberValue);
                }} else if (filterType === 'ruleId' && numberValue) {{
                    // Filter by rule ID (check if any rule in the parallel rule contains this rule ID)
                    return rule.rules.some(([draftRule, _]) => {{
                        const ruleId = draftRule[0];
                        return ruleId === parseInt(numberValue);
                    }});
                }} else if (filterType === 'ruleCount' && selectValue) {{
                    // Filter by number of rules
                    if (selectValue === '6+') {{
                        return rule.rules.length >= 6;
                    }} else {{
                        return rule.rules.length === parseInt(selectValue);
                    }}
                }}
                return true;
            }});
            
            // Update active filters display
            activeFilters = {{ filterType, value: numberValue || selectValue }};
            
            // Reset to first page and render
            currentPage = 1;
            renderPage(currentPage);
            
            // Update total count display
            const totalCountEl = document.getElementById('totalCount');
            if (filteredData.length !== data.parallel_rules.length) {{
                totalCountEl.textContent = `${{filteredData.length}} (filtered from ${{data.total_count}})`;
            }} else {{
                totalCountEl.textContent = data.total_count;
            }}
        }}
        
        function clearFilters() {{
            document.getElementById('filterNumberInput').value = '';
            document.getElementById('filterSelectInput').value = '';
            document.querySelectorAll('input[name="filterType"]')[0].checked = true;
            updateFilterInput();
            filteredData = data.parallel_rules;
            activeFilters = {{}};
            currentPage = 1;
            renderPage(currentPage);
            document.getElementById('totalCount').textContent = data.total_count;
        }}
        
        function jumpToPage() {{
            const input = document.getElementById('pageJumpInput');
            const page = parseInt(input.value);
            if (page && page >= 1 && page <= totalPages) {{
                goToPage(page);
                input.value = '';
            }}
        }}
        
        function jumpToPage2() {{
            const input = document.getElementById('pageJumpInput2');
            const page = parseInt(input.value);
            if (page && page >= 1 && page <= totalPages) {{
                goToPage(page);
                input.value = '';
            }}
        }}

        // Calculate movement grid canvas width
        const COMBINED_FRAME_SCALE = 35;
        const COMBINED_PADDING = 20;
        const gridRange = data.visibility_range * data.robots_number;
        const gridWidth = (gridRange - (-gridRange));
        const movementCanvasWidth = gridWidth * COMBINED_FRAME_SCALE + 2 * COMBINED_PADDING;
        
        // Update CSS to use calculated width for grid columns (add 30px extra space)
        const style = document.createElement('style');
        style.textContent = `
            #rulesContainer {{
                grid-template-columns: repeat(auto-fit, minmax(${{movementCanvasWidth + 30}}px, 1fr)) !important;
            }}
        `;
        document.head.appendChild(style);

        // Color mapping for robots
        const colorMap = {{
            'R': '#e74c3c',  // Red
            'G': '#2ecc71',  // Green
            'B': '#3498db',  // Blue
            'Y': '#f1c40f',  // Yellow
            'P': '#9b59b6',  // Purple
            'C': '#1abc9c',  // Cyan
            'O': '#e67e22',  // Orange
            'W': '#95a5a6',  // White/Gray
            'L': '#34495e',  // Dark Gray
            'F': '#e91e63',  // Pink
        }};

        const FRAME_SCALE = 25;
        const PADDING = 15;

        const simConfig = {{
            colors: {{ 
                L: 'red', R: 'green', F: 'blue', O: 'orange',
                r: 'red', g: 'green', b: 'blue', o: 'orange',
                B: 'blue', G: 'green', Y: 'yellow', P: 'purple',
                y: 'yellow', p: 'purple', W: '#333', w: '#333'
            }},
            boundaryColor: '#666',
            boundaryWidth: 2,
            robotRadius: 7
        }};

        function drawViewCanvas(canvasId, view, visibility) {{
            const canvas = document.getElementById(canvasId);
            if (!canvas) return;
            
            const ctx = canvas.getContext('2d');
            
            // Calculate grid bounds based on visibility: from -visibility to +visibility
            const minX = -visibility;
            const maxX = visibility;
            const minY = -visibility;
            const maxY = visibility;
            
            // Grid width/height is the number of cells between nodes
            // From -visibility to +visibility we have (visibility - (-visibility)) = 2*visibility cells
            const gridWidth = maxX - minX;
            const gridHeight = maxY - minY;
            const canvasWidth = gridWidth * FRAME_SCALE + 2 * PADDING;
            const canvasHeight = gridHeight * FRAME_SCALE + 2 * PADDING;
            
            // Set canvas dimensions - use exact size with padding for edge robots
            canvas.width = canvasWidth;
            canvas.height = canvasHeight;
            
            // Convert world coordinates to canvas coordinates
            function frameWorldToCanvas(x, y) {{
                return {{
                    cx: PADDING + (x - minX) * FRAME_SCALE,
                    cy: canvas.height - PADDING - (y - minY) * FRAME_SCALE
                }};
            }}
            
            ctx.clearRect(0, 0, canvas.width, canvas.height);
            
            // Draw grid lines only within the rhombus boundary
            ctx.strokeStyle = '#e8e8e8';
            ctx.lineWidth = 1;
            ctx.setLineDash([]);
            
            // Draw vertical lines (only segments within visibility range)
            for (let x = minX; x <= maxX; x++) {{
                for (let y = minY; y < maxY; y++) {{
                    const y1 = y;
                    const y2 = y + 1;
                    // Check if both endpoints are within the rhombus
                    const inBounds1 = Math.abs(x) + Math.abs(y1) <= visibility;
                    const inBounds2 = Math.abs(x) + Math.abs(y2) <= visibility;
                    
                    if (inBounds1 && inBounds2) {{
                        const {{ cx }} = frameWorldToCanvas(x, y1);
                        const {{ cy: cy1 }} = frameWorldToCanvas(x, y1);
                        const {{ cy: cy2 }} = frameWorldToCanvas(x, y2);
                        ctx.beginPath();
                        ctx.moveTo(cx + 0.5, cy1);
                        ctx.lineTo(cx + 0.5, cy2);
                        ctx.stroke();
                    }}
                }}
            }}
            
            // Draw horizontal lines (only segments within visibility range)
            for (let y = minY; y <= maxY; y++) {{
                for (let x = minX; x < maxX; x++) {{
                    const x1 = x;
                    const x2 = x + 1;
                    // Check if both endpoints are within the rhombus
                    const inBounds1 = Math.abs(x1) + Math.abs(y) <= visibility;
                    const inBounds2 = Math.abs(x2) + Math.abs(y) <= visibility;
                    
                    if (inBounds1 && inBounds2) {{
                        const {{ cx: cx1 }} = frameWorldToCanvas(x1, y);
                        const {{ cx: cx2 }} = frameWorldToCanvas(x2, y);
                        const {{ cy }} = frameWorldToCanvas(x1, y);
                        ctx.beginPath();
                        ctx.moveTo(cx1, cy + 0.5);
                        ctx.lineTo(cx2, cy + 0.5);
                        ctx.stroke();
                    }}
                }}
            }}
            
            // Draw small black dots on empty nodes (grid intersections)
            ctx.fillStyle = '#000';
            for (let x = minX; x <= maxX; x++) {{
                for (let y = minY; y <= maxY; y++) {{
                    const inBounds = Math.abs(x) + Math.abs(y) <= visibility;
                    if (inBounds) {{
                        const {{ cx, cy }} = frameWorldToCanvas(x, y);
                        ctx.beginPath();
                        ctx.arc(cx, cy, 2, 0, Math.PI * 2);
                        ctx.fill();
                    }}
                }}
            }}
            
            // Draw robots on nodes (grid intersections)
            view.forEach(([color, x, y]) => {{
                const {{ cx, cy }} = frameWorldToCanvas(x, y);
                const isInBounds = Math.abs(x) + Math.abs(y) <= visibility;
                
                ctx.fillStyle = simConfig.colors[color] || colorMap[color] || 'gray';
                ctx.beginPath();
                ctx.arc(cx, cy, simConfig.robotRadius, 0, Math.PI * 2);
                ctx.fill();
                
                ctx.strokeStyle = isInBounds ? '#333' : '#e53935';
                ctx.lineWidth = isInBounds ? 1.5 : 2;
                ctx.stroke();
            }});
        }}

        function drawCombinedGridCanvas(canvasId, draftRules, movableIdle, fixedIdle, robotsNumber, visibility) {{
            const canvas = document.getElementById(canvasId);
            if (!canvas) return;
            
            const ctx = canvas.getContext('2d');
            
            // Use larger scale for movement grid
            const COMBINED_FRAME_SCALE = 35;  // Bigger than the 25 used for rule views
            const COMBINED_PADDING = 20;
            
            // Square boundary: -visibility*robotsNumber to +visibility*robotsNumber
            const minX = -visibility * robotsNumber;
            const maxX = visibility * robotsNumber;
            const minY = -visibility * robotsNumber;
            const maxY = visibility * robotsNumber;
            
            const gridWidth = maxX - minX;
            const gridHeight = maxY - minY;
            const canvasWidth = gridWidth * COMBINED_FRAME_SCALE + 2 * COMBINED_PADDING;
            const canvasHeight = gridHeight * COMBINED_FRAME_SCALE + 2 * COMBINED_PADDING;
            
            canvas.width = canvasWidth;
            canvas.height = canvasHeight;
            
            function frameWorldToCanvas(x, y) {{
                return {{
                    cx: COMBINED_PADDING + (x - minX) * COMBINED_FRAME_SCALE,
                    cy: canvas.height - COMBINED_PADDING - (y - minY) * COMBINED_FRAME_SCALE
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
            
            // Draw small black dots on all nodes
            ctx.fillStyle = '#000';
            for (let x = minX; x <= maxX; x++) {{
                for (let y = minY; y <= maxY; y++) {{
                    const {{ cx, cy }} = frameWorldToCanvas(x, y);
                    ctx.beginPath();
                    ctx.arc(cx, cy, 2, 0, Math.PI * 2);
                    ctx.fill();
                }}
            }}
            
            // Draw arrows for each draft rule (rule_id, start_x, start_y, end_x, end_y)
            draftRules.forEach(([draftRule, finalRule], index) => {{
                const [ruleId, startX, startY, endX, endY] = draftRule;
                const startPos = frameWorldToCanvas(startX, startY);
                const endPos = frameWorldToCanvas(endX, endY);
                
                const finalColor = finalRule.color;
                
                // Find initial color from view at position (0,0)
                const view = finalRule.view;
                const initialColorData = view.find(([c, x, y]) => x === 0 && y === 0);
                const initialColor = initialColorData ? initialColorData[0] : finalColor;
                
                const color = simConfig.colors[finalColor] || colorMap[finalColor] || 'gray';
                
                // Check if this is an idle rule (start == end)
                const isIdle = (startX === endX && startY === endY);
                
                if (!isIdle) {{
                    // Draw arrow from start to end for moving robots
                    ctx.strokeStyle = color;
                    ctx.fillStyle = color;
                    ctx.lineWidth = 2;
                    
                    // Calculate angle and shorten the arrow to leave space before the robot
                    const angle = Math.atan2(endPos.cy - startPos.cy, endPos.cx - startPos.cx);
                    const arrowSize = 12;
                    const gapBeforeRobot = simConfig.robotRadius + 3; // Space before the robot
                    
                    // Calculate the shortened end point
                    const shortenedEndX = endPos.cx - gapBeforeRobot * Math.cos(angle);
                    const shortenedEndY = endPos.cy - gapBeforeRobot * Math.sin(angle);
                    
                    // Draw arrow line (from start to shortened end)
                    ctx.beginPath();
                    ctx.moveTo(startPos.cx, startPos.cy);
                    ctx.lineTo(shortenedEndX, shortenedEndY);
                    ctx.stroke();
                    
                    // Draw arrowhead at shortened end position
                    ctx.beginPath();
                    ctx.moveTo(shortenedEndX, shortenedEndY);
                    ctx.lineTo(
                        shortenedEndX - arrowSize * Math.cos(angle - Math.PI / 6),
                        shortenedEndY - arrowSize * Math.sin(angle - Math.PI / 6)
                    );
                    ctx.lineTo(
                        shortenedEndX - arrowSize * Math.cos(angle + Math.PI / 6),
                        shortenedEndY - arrowSize * Math.sin(angle + Math.PI / 6)
                    );
                    ctx.closePath();
                    ctx.fill();
                }}
                
                // Draw robot at starting position with initial color
                const initColor = simConfig.colors[initialColor] || colorMap[initialColor] || 'gray';
                ctx.fillStyle = initColor;
                ctx.beginPath();
                ctx.arc(startPos.cx, startPos.cy, simConfig.robotRadius, 0, Math.PI * 2);
                ctx.fill();
                ctx.strokeStyle = '#333';
                ctx.lineWidth = 1.5;
                ctx.stroke();
                
                // For idle robots, draw a curved arrow (half circle loop)
                if (isIdle) {{
                    const loopRadius = 8;
                    const loopCenterX = startPos.cx + loopRadius + 2;
                    const loopCenterY = startPos.cy - loopRadius;
                    
                    // Draw the arc (half circle from top to right)
                    ctx.strokeStyle = color;
                    ctx.fillStyle = color;
                    ctx.lineWidth = 2;
                    ctx.beginPath();
                    ctx.arc(loopCenterX, loopCenterY, loopRadius, Math.PI * 0.75, Math.PI * 2.25, false);
                    ctx.stroke();
                    
                    // Draw arrowhead at the end of the arc (pointing back to start)
                    const arrowAngle = Math.PI * 2.25; // End angle of arc
                    const arrowEndX = loopCenterX + loopRadius * Math.cos(arrowAngle);
                    const arrowEndY = loopCenterY + loopRadius * Math.sin(arrowAngle);
                    const arrowSize = 6;
                    
                    // Arrow pointing in the direction of the arc
                    const tangentAngle = arrowAngle + Math.PI / 2; // Tangent to circle
                    ctx.beginPath();
                    ctx.moveTo(arrowEndX, arrowEndY);
                    ctx.lineTo(
                        arrowEndX - arrowSize * Math.cos(tangentAngle - Math.PI / 6),
                        arrowEndY - arrowSize * Math.sin(tangentAngle - Math.PI / 6)
                    );
                    ctx.lineTo(
                        arrowEndX - arrowSize * Math.cos(tangentAngle + Math.PI / 6),
                        arrowEndY - arrowSize * Math.sin(tangentAngle + Math.PI / 6)
                    );
                    ctx.closePath();
                    ctx.fill();
                }}
                ctx.stroke();
            }});
            
            // Draw idle robots (they don't move)
            const allIdle = [...movableIdle, ...fixedIdle];
            allIdle.forEach(([color, x, y]) => {{
                const {{ cx, cy }} = frameWorldToCanvas(x, y);
                
                ctx.fillStyle = simConfig.colors[color] || colorMap[color] || 'gray';
                ctx.beginPath();
                ctx.arc(cx, cy, simConfig.robotRadius, 0, Math.PI * 2);
                ctx.fill();
                
                ctx.strokeStyle = '#333';
                ctx.lineWidth = 1.5;
                ctx.stroke();
            }});
        }}

        function renderCombinedGrid(rule, robotsNumber, visibility) {{
            const canvasId = `combinedCanvas_${{rule.id}}`;
            
            let html = '<div class="rule-section">';
            html += `<div style="margin-top: 0px;"><canvas class="view-canvas" id="${{canvasId}}"></canvas></div>`;
            html += '</div>';
            
            // Store canvas data for later drawing
            if (!window.pendingCanvases) window.pendingCanvases = [];
            window.pendingCanvases.push({{
                canvasId: canvasId,
                draftRules: rule.rules,
                movableIdle: rule.movable_idle_robots,
                fixedIdle: rule.fixed_idle_robots,
                robotsNumber: robotsNumber,
                visibility: visibility,
                isCombined: true
            }});
            
            return html;
        }}

        function renderParallelRule(rule) {{
            let html = `<div class="rule-card">`;
            html += `<div class="rule-header">ParallelRule ${{rule.id}} (${{rule.rules.length}} rules)</div>`;
            
            // Combined Grid - shows movements with arrows (at the top)
            html += renderCombinedGrid(
                rule,
                data.robots_number,
                data.visibility_range
            );
            
            // Final Rules Details - Horizontal Layout with Canvas
            html += `<div class="rule-section"><strong>Rules Details:</strong>`;
            html += `<div class="rules-list">`;
            rule.rules.forEach(([draftRule, finalRule]) => {{
                const [ruleId, startX, startY, endX, endY] = draftRule;
                const canvasId = `viewCanvas_${{rule.id}}_${{ruleId}}`;
                html += `<div class="rule-detail-card">`;
                html += `<h6>Rule ${{ruleId}}</h6>`;
                html += `<canvas class="view-canvas" id="${{canvasId}}"></canvas>`;
                html += `<div><strong>Direction:</strong> ${{finalRule.direction}}</div>`;
                html += `<div><strong>Color:</strong> '${{finalRule.color}}'</div>`;
                html += `</div>`;
            }});
            html += `</div></div>`;
            
            // Idle Robots
            if (rule.movable_idle_robots.length > 0 || rule.fixed_idle_robots.length > 0) {{
                html += `<div class="rule-section">`;
                if (rule.fixed_idle_robots.length > 0) {{
                    html += `<div><strong>Idles:</strong> [${{rule.fixed_idle_robots.map(([c,x,y]) => `${{c}}(${{x}},${{y}})`).join(', ')}}]</div>`;
                }}
                if (rule.movable_idle_robots.length > 0) {{
                    html += `<div><strong>Movables:</strong> [${{rule.movable_idle_robots.map(([c,x,y]) => `${{c}}(${{x}},${{y}})`).join(', ')}}]</div>`;
                }}
                html += `</div>`;
            }}
            
            // Positions
            html += `<div class="rule-section">`;
            html += `<div><strong>Start:</strong> [${{rule.starting_positions.map(([c,x,y]) => `${{c}}(${{x}},${{y}})`).join(', ')}}]</div>`;
            html += `<div><strong>End:</strong> [${{rule.ending_positions.map(([c,x,y]) => `${{c}}(${{x}},${{y}})`).join(', ')}}]</div>`;
            html += `</div>`;
            
            // Stats - compact single line
            html += `<div class="rule-section" style="font-weight: bold;">`;
            html += `${{rule.active_color_count}} act colors | ${{rule.active_movement_count}} act movements`;
            html += `</div>`;
            
            html += `</div>`;
            
            // Store rule data for later canvas drawing
            if (!window.pendingCanvases) window.pendingCanvases = [];
            rule.rules.forEach(([draftRule, finalRule]) => {{
                const [ruleId, startX, startY, endX, endY] = draftRule;
                const canvasId = `viewCanvas_${{rule.id}}_${{ruleId}}`;
                window.pendingCanvases.push({{
                    canvasId: canvasId,
                    view: finalRule.view,
                    visibility: data.visibility_range
                }});
            }});
            
            return html;
        }}

        // Pagination
        const ITEMS_PER_PAGE = 100;
        let currentPage = 1;
        let totalPages = Math.ceil(filteredData.length / ITEMS_PER_PAGE);

        function renderPage(page) {{
            const container = document.getElementById('rulesContainer');
            container.innerHTML = '';
            window.pendingCanvases = []; // Reset pending canvases
            
            // Recalculate total pages based on filtered data
            totalPages = Math.ceil(filteredData.length / ITEMS_PER_PAGE);
            
            const start = (page - 1) * ITEMS_PER_PAGE;
            const end = Math.min(start + ITEMS_PER_PAGE, filteredData.length);
            
            for (let i = start; i < end; i++) {{
                container.innerHTML += renderParallelRule(filteredData[i]);
            }}
            
            // Draw all canvases after DOM is ready
            setTimeout(() => {{
                if (window.pendingCanvases) {{
                    window.pendingCanvases.forEach((item) => {{
                        if (item.isCombined) {{
                            // Draw combined grid with arrows
                            drawCombinedGridCanvas(item.canvasId, item.draftRules, item.movableIdle, item.fixedIdle, item.robotsNumber, item.visibility);
                        }} else {{
                            // Draw view canvas (rule view)
                            drawViewCanvas(item.canvasId, item.view, item.visibility);
                        }}
                    }});
                }}
            }}, 50);
            
            // Update pagination info (both top and bottom)
            const pageText = `Page ${{page}} of ${{totalPages}} (Showing ${{start + 1}}-${{end}} of ${{filteredData.length}})`;
            document.getElementById('pageInfo').textContent = pageText;
            document.getElementById('pageInfo2').textContent = pageText;
            
            // Update max for page jump inputs
            document.getElementById('pageJumpInput').max = totalPages;
            document.getElementById('pageJumpInput2').max = totalPages;
            
            // Update button states
            document.getElementById('prevBtn').disabled = page === 1;
            document.getElementById('nextBtn').disabled = page === totalPages;
            document.getElementById('firstBtn').disabled = page === 1;
            document.getElementById('lastBtn').disabled = page === totalPages;
            
            // Scroll to top
            window.scrollTo({{ top: 0, behavior: 'smooth' }});
        }}

        function goToPage(page) {{
            if (page >= 1 && page <= totalPages) {{
                currentPage = page;
                renderPage(currentPage);
            }}
        }}

        // Initialize with error handling
        try {{
            console.log('Initializing page...');
            renderPage(currentPage);
            console.log('Page initialized successfully');
        }} catch (error) {{
            console.error('Error initializing page:', error);
            document.getElementById('rulesContainer').innerHTML = 
                '<div class="alert alert-danger">Error loading parallel rules: ' + error.message + '</div>';
        }}
    </script>
</body>
</html>"#,
        json_data = json_data,
        number_of_robots = number_of_robots
    );

    fs::write(output_path, html).expect("Failed to write HTML file");
    println!("✅ Parallel rules HTML generated at: {}", output_path);
}
