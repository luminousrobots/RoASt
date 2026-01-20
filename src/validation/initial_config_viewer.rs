use std::fs;
use std::io::Write;

use crate::validation::initial_config_generator::InitialConfig;

/// Generate an HTML file that displays initial configurations
///
/// # Arguments
/// * `initial_configurations` - List of initial configurations from config
/// * `visibility_range` - Visibility range for robots
/// * `output_path` - Path where to save the HTML file
pub fn initial_config_viewer_html(
    initial_configurations: Vec<InitialConfig>,
    visibility_range: i16,
    is_obstacle_opaque: bool,
    number_of_robots: usize,
    output_path: &str,
) -> std::io::Result<()> {
    let html = create_html_content(
        &initial_configurations,
        visibility_range,
        is_obstacle_opaque,
        number_of_robots,
    );

    let mut file = fs::File::create(output_path)?;
    file.write_all(html.as_bytes())?;

    println!("Viewer HTML generated at: {}", output_path);
    Ok(())
}

fn create_html_content(
    initial_configurations: &[InitialConfig],
    visibility: i16,
    is_obstacle_opaque: bool,
    num_robots: usize,
) -> String {
    let initial_configs_json = format_collected_configs_json(initial_configurations);

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Collected Configs Viewer</title>
    <script src="https://cdn.tailwindcss.com"></script>
    <style>
        body {{
            font-family: "Inter", sans-serif;
        }}
        #page-container {{
            --cell-size: 30px;
            --grid-gap: 1.5rem;
        }}
        #output-container {{
            display: grid;
            grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
            gap: var(--grid-gap);
            padding: 1.5rem;
            background-color: #f9fafb;
            border-radius: 0.5rem;
            border: 1px solid #e5e7eb;
            margin-top: 1.5rem;
        }}
        .grid-wrapper {{
            background-color: #ffffff;
            border-radius: 0.5rem;
            border: 1px solid #e9ecef;
            padding: 1rem;
            box-shadow: 0 1px 3px 0 rgba(0, 0, 0, 0.1), 0 1px 2px 0 rgba(0, 0, 0, 0.06);
            display: flex;
            flex-direction: column;
            align-items: center;
        }}
        .grid-wrapper.leader-config {{
            border: 3px solid #dc3545;
            box-shadow: 0 0 10px rgba(220, 53, 69, 0.3);
        }}
        .grid-wrapper.excluded {{
            opacity: 0.3;
            position: relative;
        }}
        .grid-wrapper.excluded::after {{
            content: '✗';
            position: absolute;
            top: 50%;
            left: 50%;
            transform: translate(-50%, -50%);
            font-size: 5rem;
            color: #dc3545;
            font-weight: 900;
            pointer-events: none;
        }}
        .grid-title {{
            font-size: 0.9rem;
            font-weight: 700;
            text-align: center;
            margin-bottom: 0.75rem;
            font-family: 'Inter', sans-serif;
        }}
        .leader-marker {{
            color: #dc3545;
            font-weight: 900;
            margin-left: 0.25rem;
            font-size: 1.5rem;
        }}
        .view-canvas {{
            border: 1px solid #dee2e6;
            border-radius: 4px;
            background-color: #f8f9fa;
        }}
    </style>
</head>
<body class="bg-gray-100">
    <div class="container mx-auto p-4 sm:p-8">
        <div class="bg-white p-6 sm:p-10 rounded-2xl shadow-lg w-full max-w-7xl mx-auto">
            <h1 class="text-3xl font-bold text-gray-900 mb-2">Initial Configurations Viewer</h1>
            <div class="mb-6 text-sm text-gray-600">
                <p><strong>Visibility Range:</strong> {visibility}</p>
                <p><strong>Obstacle Opaque:</strong> {is_obstacle_opaque}</p>
                <p><strong>Initial Configurations:</strong> {initial_configs_count}</p>
            </div>

            <div id="page-container">
                <div id="output-container"></div>
            </div>
        </div>
    </div>

    <script>
        const InitialConfigs = {initial_configs_json};
        const visibility = {visibility};
        const isObstacleOpaque = {is_obstacle_opaque};
        const numRobots = {num_robots};

        const letterColorMap = {{
            'L': '#FF0000', 'R': '#008000', 'F': '#0000FF', 'O': '#FFA500',
            'r': '#FF0000', 'g': '#008000', 'b': '#0000FF', 'o': '#FFA500',
            'B': '#0000FF', 'G': '#008000', 'Y': '#FFFF00', 'P': '#800080',
            'y': '#FFFF00', 'p': '#800080', 'W': '#333333', 'w': '#333333',
        }};

        function createGrid(config, visibility, title, standardBounds, isLeader = false, isExcluded = false) {{
            const wrapper = document.createElement('div');
            wrapper.className = 'grid-wrapper';
            if (isLeader) wrapper.classList.add('leader-config');
            if (isExcluded) wrapper.classList.add('excluded');

            const titleElement = document.createElement('h3');
            titleElement.className = 'grid-title';
            titleElement.textContent = title;
            if (isLeader) {{
                const marker = document.createElement('span');
                marker.className = 'leader-marker';
                marker.textContent = ' *';
                titleElement.appendChild(marker);
            }}
            wrapper.appendChild(titleElement);
            
            const min_x = standardBounds.min_x;
            const max_x = standardBounds.max_x;
            const min_y = standardBounds.min_y;
            const max_y = standardBounds.max_y;

            const width = max_x - min_x;
            const height = max_y - min_y;
            
            const canvas = document.createElement('canvas');
            canvas.className = 'view-canvas';
            const cellSize = 25;
            const padding = 30;
            canvas.width = width * cellSize + padding * 2;
            canvas.height = height * cellSize + padding * 2;
            
            const ctx = canvas.getContext('2d');
            const offsetX = padding;
            const offsetY = padding;
            
            const worldToCanvas = (wx, wy) => ({{
                cx: offsetX + (wx - min_x) * cellSize,
                cy: offsetY + (max_y - wy) * cellSize
            }});
            
            // Draw grid lines
            ctx.strokeStyle = '#e8e8e8';
            ctx.lineWidth = 1;
            for (let i = 0; i <= width; i++) {{
                const x = offsetX + i * cellSize;
                ctx.beginPath();
                ctx.moveTo(x, offsetY);
                ctx.lineTo(x, offsetY + height * cellSize);
                ctx.stroke();
            }}
            for (let i = 0; i <= height; i++) {{
                const y = offsetY + i * cellSize;
                ctx.beginPath();
                ctx.moveTo(offsetX, y);
                ctx.lineTo(offsetX + width * cellSize, y);
                ctx.stroke();
            }}
            
            // Draw coordinate labels
            ctx.fillStyle = '#666';
            ctx.font = '10px Arial';
            ctx.textAlign = 'center';
            ctx.textBaseline = 'middle';
            
            // X-axis labels (bottom)
            for (let wx = min_x; wx <= max_x; wx++) {{
                const {{ cx }} = worldToCanvas(wx, min_y);
                ctx.fillText(wx, cx, offsetY + height * cellSize + 15);
            }}
            
            // Y-axis labels (left)
            ctx.textAlign = 'right';
            for (let wy = min_y; wy <= max_y; wy++) {{
                const {{ cy }} = worldToCanvas(min_x, wy);
                ctx.fillText(wy, offsetX - 10, cy);
            }}
            
            ctx.textAlign = 'center';
            ctx.textBaseline = 'middle';
            
            // Draw visibility range
            const robots = config.filter(([ch]) => ch !== 'O');
            ctx.strokeStyle = '#87ceeb';
            ctx.lineWidth = 1;
            ctx.setLineDash([3, 3]);
            
            robots.forEach(([ch, rx, ry]) => {{
                const {{ cx, cy }} = worldToCanvas(rx, ry);
                const visibilityPx = visibility * cellSize;
                
                ctx.beginPath();
                ctx.moveTo(cx, cy - visibilityPx);
                ctx.lineTo(cx + visibilityPx, cy);
                ctx.lineTo(cx, cy + visibilityPx);
                ctx.lineTo(cx - visibilityPx, cy);
                ctx.closePath();
                ctx.stroke();
            }});
            
            ctx.setLineDash([]);
            
            // Draw robots and obstacle
            config.forEach(([ch, x, y]) => {{
                const {{ cx, cy }} = worldToCanvas(x, y);
                
                ctx.beginPath();
                ctx.arc(cx, cy, 8, 0, 2 * Math.PI);
                
                const hexColor = letterColorMap[ch] || '#808080';
                ctx.fillStyle = hexColor;
                ctx.fill();
                
                ctx.strokeStyle = '#000';
                ctx.lineWidth = 2;
                ctx.stroke();
                
                ctx.fillStyle = '#fff';
                ctx.font = 'bold 12px Arial';
                ctx.textAlign = 'center';
                ctx.textBaseline = 'middle';
                ctx.fillText(ch, cx, cy);
            }});
            
            wrapper.appendChild(canvas);
            return wrapper;
        }}

        function calculateStandardBounds(numRobots, visibility) {{
            const gridRange = numRobots * visibility;
            return {{
                min_x: -gridRange,
                max_x: gridRange,
                min_y: -gridRange,
                max_y: gridRange
            }};
        }}

        function renderAll() {{
            const outputDiv = document.getElementById('output-container');
            outputDiv.innerHTML = '';

            // Use fixed bounds based on numRobots * visibility
            const standardBounds = calculateStandardBounds(numRobots, visibility);

            // Render initial configurations
            InitialConfigs.forEach(([config, isLeader], i) => {{
                const isExcluded = false;
                const grid = createGrid(config, visibility, `Config ${{i}}`, standardBounds, isLeader, isExcluded);
                outputDiv.appendChild(grid);
            }});
        }}

        renderAll();
    </script>
</body>
</html>"#,
        visibility = visibility,
        is_obstacle_opaque = is_obstacle_opaque,
        initial_configs_count = initial_configurations.len(),
        initial_configs_json = initial_configs_json,
    )
}

fn format_collected_configs_json(configs: &[InitialConfig]) -> String {
    let configs_str = configs
        .iter()
        .map(|(config, is_leader)| {
            let positions = config
                .iter()
                .map(|(ch, x, y)| format!("['{}', {}, {}]", ch, x, y))
                .collect::<Vec<_>>()
                .join(", ");
            format!("[[{}], {}]", positions, is_leader)
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{}]", configs_str)
}
