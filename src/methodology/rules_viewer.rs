use serde::{Deserialize, Serialize};

use crate::{
    methodology::globals::{
        get_number_of_colors, get_number_of_robots, get_original_rules_count, get_rules, get_views,
        get_visibility,
    },
    modules::final_rule::FinalRule,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleInfo {
    pub id: usize,
    pub rule: FinalRule,
    pub is_original: bool, // true if id < original_rules_count
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RulesCollection {
    pub rules: Vec<RuleInfo>,
    pub total_count: usize,
    pub original_rules_count: usize,
    pub visibility_range: i16,
    pub robots_number: usize,
    pub colors_number: usize,
}

pub fn create_rules_collection() -> RulesCollection {
    let original_count = get_original_rules_count();
    let mut rules_info = Vec::new();

    for (i, rule) in get_rules().iter().enumerate() {
        rules_info.push(RuleInfo {
            id: i,
            rule: FinalRule {
                view: get_views()[rule.view_id].clone(),
                direction: rule.direction,
                color: rule.color,
            },
            is_original: i < original_count,
        });
    }

    RulesCollection {
        total_count: rules_info.len(),
        original_rules_count: original_count,
        rules: rules_info,
        visibility_range: get_visibility().clone(),
        robots_number: *get_number_of_robots(),
        colors_number: *get_number_of_colors(),
    }
}

pub fn generate_rules_html(collection: &RulesCollection, output_path: &str) {
    use std::fs;

    let json_data = serde_json::to_string(collection).expect("Failed to serialize");

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Rules Viewer</title>
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
        
        .rule-card.new-rule {{
            border-left: 4px solid #3498db;
        }}
        
        .rule-card.original-rule {{
            border-left: 4px solid #2ecc71;
        }}
        
        .rule-header {{
            background: #2c3e50;
            color: white;
            padding: 8px 12px;
            margin: -12px -12px 12px -12px;
            border-radius: 4px 4px 0 0;
            font-weight: 600;
            font-size: 13px;
            display: flex;
            justify-content: space-between;
            align-items: center;
        }}
        
        .rule-badge {{
            background: rgba(255,255,255,0.2);
            padding: 2px 8px;
            border-radius: 3px;
            font-size: 11px;
            font-weight: 500;
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
        
        .view-canvas {{
            border: 1px solid #e0e0e0;
            border-radius: 2px;
            margin: 6px 0;
            background: white;
            display: block;
            max-width: 100%;
            height: auto;
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
    </style>
</head>
<body>
    <div class="container-fluid">
        <h1>Rules Viewer</h1>
        <div class="alert alert-info">
            <strong>Total: <span id="totalCount"></span></strong> | 
            <strong>Original: <span id="originalCount"></span></strong> | 
            <strong>New: <span id="newCount"></span></strong> | 
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
                        <input type="radio" name="filterType" value="all" checked onchange="updateFilterInput()"> All Rules
                    </label>
                    <label style="font-weight: normal; cursor: pointer; font-size: 13px;">
                        <input type="radio" name="filterType" value="original" onchange="updateFilterInput()"> Original Rules Only
                    </label>
                    <label style="font-weight: normal; cursor: pointer; font-size: 13px;">
                        <input type="radio" name="filterType" value="new" onchange="updateFilterInput()"> New Rules Only
                    </label>
                    <label style="font-weight: normal; cursor: pointer; font-size: 13px;">
                        <input type="radio" name="filterType" value="ruleId" onchange="updateFilterInput()"> Rule ID
                    </label>
                </div>
            </div>
            <div class="filter-row" id="ruleIdRow" style="display: none;">
                <div class="filter-group">
                    <label>Rule ID:</label>
                    <input type="number" id="filterRuleId" placeholder="e.g., 42" min="0" oninput="applyFilters()">
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
                    <input type="number" id="pageJumpInput" min="1" placeholder="Page" onkeypress="if(event.key==='Enter') jumpToPage()">
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
                    <input type="number" id="pageJumpInput2" min="1" placeholder="Page" onkeypress="if(event.key==='Enter') jumpToPage2()">
                    <button onclick="jumpToPage2()">Go</button>
                </div>
            </div>
        </div>
    </div>

    <script>
        const data = {json_data};
        
        console.log('Data loaded:', data);
        console.log('Total rules:', data.rules.length);
        
        const newRulesCount = data.total_count - data.original_rules_count;
        document.getElementById('totalCount').textContent = data.total_count;
        document.getElementById('originalCount').textContent = data.original_rules_count;
        document.getElementById('newCount').textContent = newRulesCount;
        document.getElementById('visibility').textContent = data.visibility_range;
        document.getElementById('robotsNumber').textContent = data.robots_number;
        document.getElementById('colorsNumber').textContent = data.colors_number;
        
        // Calculate rule view canvas width and update grid
        const FRAME_SCALE = 25;
        const PADDING = 15;
        const visibility = data.visibility_range;
        const gridWidth = (visibility - (-visibility));
        const gridHeight = (visibility - (-visibility));
        const canvasWidth = gridWidth * FRAME_SCALE + 2 * PADDING;
        
        // Update CSS to use calculated width for grid columns
        const style = document.createElement('style');
        style.textContent = `
            #rulesContainer {{
                grid-template-columns: repeat(auto-fit, minmax(${{canvasWidth + 30}}px, 1fr)) !important;
            }}
        `;
        document.head.appendChild(style);
        
        // Filtering
        let filteredData = data.rules;
        
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
            const ruleIdRow = document.getElementById('ruleIdRow');
            const ruleIdInput = document.getElementById('filterRuleId');
            
            if (filterType === 'ruleId') {{
                ruleIdRow.style.display = 'flex';
                ruleIdInput.value = '';
            }} else {{
                ruleIdRow.style.display = 'none';
                ruleIdInput.value = '';
            }}
            
            applyFilters();
        }}
        
        function applyFilters() {{
            const filterType = document.querySelector('input[name="filterType"]:checked').value;
            const ruleIdValue = document.getElementById('filterRuleId').value;
            
            filteredData = data.rules.filter(ruleInfo => {{
                if (filterType === 'all') {{
                    return true;
                }} else if (filterType === 'original') {{
                    return ruleInfo.is_original;
                }} else if (filterType === 'new') {{
                    return !ruleInfo.is_original;
                }} else if (filterType === 'ruleId' && ruleIdValue) {{
                    return ruleInfo.id === parseInt(ruleIdValue);
                }}
                return true;
            }});
            
            // Reset to first page and render
            currentPage = 1;
            renderPage(currentPage);
            
            // Update total count display
            const totalCountEl = document.getElementById('totalCount');
            if (filteredData.length !== data.rules.length) {{
                totalCountEl.textContent = `${{filteredData.length}} (filtered from ${{data.total_count}})`;
            }} else {{
                totalCountEl.textContent = data.total_count;
            }}
        }}
        
        function clearFilters() {{
            document.getElementById('filterRuleId').value = '';
            document.querySelectorAll('input[name="filterType"]')[0].checked = true;
            updateFilterInput();
            filteredData = data.rules;
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

        const simConfig = {{
            colors: {{ 
                L: 'red', R: 'green', F: 'blue', O: 'orange',
                r: 'red', g: 'green', b: 'blue', o: 'orange',
                B: 'blue', G: 'green', Y: 'yellow', P: 'purple',
                y: 'yellow', p: 'purple', W: '#333', w: '#333'
            }},
            boundaryColor: '#666',
            boundaryWidth: 2,
            robotRadius: 6
        }};

        function drawViewCanvas(canvasId, view, visibility) {{
            const canvas = document.getElementById(canvasId);
            if (!canvas) return;
            
            const ctx = canvas.getContext('2d');
            
            const minX = -visibility;
            const maxX = visibility;
            const minY = -visibility;
            const maxY = visibility;
            
            const gridWidth = maxX - minX;
            const gridHeight = maxY - minY;
            const canvasWidth = gridWidth * FRAME_SCALE + 2 * PADDING;
            const canvasHeight = gridHeight * FRAME_SCALE + 2 * PADDING;
            
            canvas.width = canvasWidth;
            canvas.height = canvasHeight;
            
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
            
            // Draw vertical lines
            for (let x = minX; x <= maxX; x++) {{
                for (let y = minY; y < maxY; y++) {{
                    const y1 = y;
                    const y2 = y + 1;
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
            
            // Draw horizontal lines
            for (let y = minY; y <= maxY; y++) {{
                for (let x = minX; x < maxX; x++) {{
                    const x1 = x;
                    const x2 = x + 1;
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
            
            // Draw small black dots on empty nodes
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
            
            // Draw robots on nodes
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

        function renderRule(ruleInfo) {{
            const rule = ruleInfo.rule;
            const isOriginal = ruleInfo.is_original;
            const ruleClass = isOriginal ? 'original-rule' : 'new-rule';
            const badgeText = isOriginal ? 'ORIGINAL' : 'NEW';
            const canvasId = `viewCanvas_${{ruleInfo.id}}`;
            
            let html = `<div class="rule-card ${{ruleClass}}">`;
            html += `<div class="rule-header">`;
            html += `<span>Rule ${{ruleInfo.id}}</span>`;
            html += `<span class="rule-badge">${{badgeText}}</span>`;
            html += `</div>`;
            
            // View Canvas
            html += `<div class="rule-section">`;
            html += `<canvas class="view-canvas" id="${{canvasId}}"></canvas>`;
            html += `</div>`;
            
            // Rule Details
            html += `<div class="rule-section">`;
            html += `<div><strong>Direction:</strong> ${{rule.direction}}</div>`;
            html += `<div><strong>Color:</strong> '${{rule.color}}'</div>`;
            html += `</div>`;
            
            html += `</div>`;
            
            // Store canvas data for later drawing
            if (!window.pendingCanvases) window.pendingCanvases = [];
            window.pendingCanvases.push({{
                canvasId: canvasId,
                view: rule.view,
                visibility: data.visibility_range
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
            window.pendingCanvases = [];
            
            totalPages = Math.ceil(filteredData.length / ITEMS_PER_PAGE);
            
            const start = (page - 1) * ITEMS_PER_PAGE;
            const end = Math.min(start + ITEMS_PER_PAGE, filteredData.length);
            
            for (let i = start; i < end; i++) {{
                container.innerHTML += renderRule(filteredData[i]);
            }}
            
            // Draw all canvases after DOM is ready
            setTimeout(() => {{
                if (window.pendingCanvases) {{
                    window.pendingCanvases.forEach((item) => {{
                        drawViewCanvas(item.canvasId, item.view, item.visibility);
                    }});
                }}
            }}, 50);
            
            // Update pagination info
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

        // Initialize
        try {{
            console.log('Initializing page...');
            renderPage(currentPage);
            console.log('Page initialized successfully');
        }} catch (error) {{
            console.error('Error initializing page:', error);
            document.getElementById('rulesContainer').innerHTML = 
                '<div class="alert alert-danger">Error loading rules: ' + error.message + '</div>';
        }}
    </script>
</body>
</html>"#,
        json_data = json_data
    );

    fs::write(output_path, html).expect("Failed to write HTML file");
    println!("✅ Rules HTML generated at: {}", output_path);
}
