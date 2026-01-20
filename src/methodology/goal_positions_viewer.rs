use std::fs;
use std::path::Path;

/// Generates an HTML page with embedded position data JSON for goal/target positions
pub fn generate_goal_positions_viewer(
    positions: &[Vec<Vec<(char, i16, i16)>>],
    executions: &[Vec<usize>],
    output_dir: &str,
    goal_index: usize,
    target_index: usize,
) -> String {
    let filename = format!(
        "goal_{}_target_{}_positions_viewer.html",
        goal_index, target_index
    );
    let filepath = Path::new(output_dir).join(&filename);

    // Convert positions and executions to JSON strings
    let positions_json =
        serde_json::to_string(positions).expect("Failed to serialize positions to JSON");
    let executions_json =
        serde_json::to_string(executions).expect("Failed to serialize executions to JSON");

    let html_content =
        generate_html_template(&positions_json, &executions_json, goal_index, target_index);

    fs::write(&filepath, html_content).unwrap_or_else(|e| {
        panic!(
            "Failed to write goal positions viewer to {:?}: {}",
            filepath, e
        )
    });

    println!("✅ Generated goal positions viewer: {}", filename);
    filename.to_string()

    // filepath.to_string_lossy().to_string()
}

fn generate_html_template(
    positions_json: &str,
    executions_json: &str,
    goal_index: usize,
    target_index: usize,
) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Position Viewer - Goal {} Target {}</title>
    <link
      href="https://cdn.jsdelivr.net/npm/bootstrap@5.3.0/dist/css/bootstrap.min.css"
      rel="stylesheet"
    />
    <link
      rel="stylesheet"
      href="https://cdn.jsdelivr.net/npm/bootstrap-icons@1.10.5/font/bootstrap-icons.css"
    />
    <link rel="preconnect" href="https://fonts.googleapis.com" />
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin />
    <link
      href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;700&display=swap"
      rel="stylesheet"
    />
    <style>
      body {{
        background-color: #f0f2f5;
        font-family: "Inter", sans-serif;
      }}
      .container {{
        max-width: 1200px;
      }}
      .canvas-wrapper {{
        background-color: #fff;
        border-radius: 12px;
        padding: 1rem;
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.08);
        transition: transform 0.2s ease-in-out;
        display: flex;
        flex-direction: column;
        align-items: center;
        width: 100%;
      }}
      .canvas-wrapper:hover {{
        transform: translateY(-5px);
      }}
      canvas {{
        width: 100%;
        height: auto;
        border-radius: 8px;
        position: relative;
        background-color: #fdfdfd;
      }}
      .execution-title {{
        font-weight: 500;
        color: #495057;
        font-size: 1.1rem;
      }}
      #executionCount {{
        background: white;
        padding: 8px 15px;
        border: 1px solid #dee2e6;
        border-radius: 8px;
        font-weight: 500;
        font-size: 1.1rem;
        color: #495057;
        white-space: nowrap;
      }}
      .header-controls {{
        padding: 1rem 1.5rem;
        background-color: #ffffff;
        border-radius: 12px;
        margin-bottom: 2rem;
        box-shadow: 0 2px 8px rgba(0, 0, 0, 0.06);
      }}
      #q1-title {{
        font-size: 1.5rem;
        font-weight: 700;
        color: #212529;
      }}
      .btn-group .btn {{
        margin: 0 2px;
      }}
      .btn:disabled {{
        opacity: 0.5;
        pointer-events: none;
      }}
      .parallel-rules-wrapper {{
        width: 100%;
        margin-top: 1rem;
        padding-top: 1rem;
        border-top: 1px solid #e9ecef;
        font-size: 0.9rem;
        color: #212529;
        word-wrap: break-word;
        text-align: left;
      }}
    </style>
  </head>
  <body>
    <div class="container mt-4">
      <div class="row header-controls align-items-center gy-3">
        <div class="col-md-6 text-center text-md-start">
          <h2 class="mb-0" id="q1-title">Position Viewer - Goal {} Target {}</h2>
        </div>
        <div class="col-md-6">
          <div class="d-flex justify-content-center justify-content-md-end align-items-center">
            <div id="executionCount" class="me-3">0 executions!</div>
            <div class="btn-group me-2" role="group">
              <button id="previousAll" class="btn btn-outline-secondary" title="Previous All">
                <i class="bi bi-skip-backward-fill"></i>
              </button>
              <button id="pauseResumeAll" class="btn btn-outline-secondary" title="Pause/Resume All">
                <i class="bi bi-play-fill"></i>
              </button>
              <button id="nextAll" class="btn btn-outline-secondary" title="Next All">
                <i class="bi bi-skip-forward-fill"></i>
              </button>
            </div>
            <button id="replayAll" class="btn btn-outline-primary" title="Replay All">
              <i class="bi bi-arrow-clockwise"></i>
            </button>
          </div>
        </div>
      </div>

      <div class="row justify-content-center" id="canvasContainer"></div>
    </div>
    <script>
      const EMBEDDED_POSITIONS = {};
      const EMBEDDED_EXECUTIONS = {};

      const config = {{
        colors: {{ L: "red", R: "green", F: "blue", Y: "yellow", O: "orange" }},
        data: EMBEDDED_POSITIONS,
        executions: EMBEDDED_EXECUTIONS,
        animations: {{}},
        executionCount: 0,
      }};

      document.addEventListener('DOMContentLoaded', () => {{
        config.executionCount = config.data.length;
        document.getElementById("executionCount").innerText = config.executionCount + ' execution' + (config.executionCount === 1 ? '' : 's') + '!';

        document.getElementById("replayAll").addEventListener("click", () => setupAndPlaySequences(true));
        document.getElementById("pauseResumeAll").addEventListener("click", togglePauseResumeAll);
        document.getElementById("previousAll").addEventListener("click", previousAllSteps);
        document.getElementById("nextAll").addEventListener("click", nextAllSteps);

        if (config.executionCount > 0) {{
          setupAndPlaySequences(false);
        }}
      }});

      function createCanvas(id) {{
        const col = document.createElement("div");
        col.className = "col-12 col-md-6 col-lg-4 mb-4";

        const wrapper = document.createElement("div");
        wrapper.className = "canvas-wrapper";

        const cardHeader = document.createElement("div");
        cardHeader.className = "d-flex justify-content-between align-items-center w-100 mb-3";

        const titleContainer = document.createElement("div");
        titleContainer.className = "d-flex flex-column";

        const title = document.createElement("h5");
        title.className = "execution-title m-0";
        title.innerText = 'Execution ' + (id + 1);

        const stepInfo = document.createElement("small");
        stepInfo.className = "text-muted";
        stepInfo.id = 'step-info-' + id;
        stepInfo.innerText = "Step 0/0";

        titleContainer.appendChild(title);
        titleContainer.appendChild(stepInfo);
        cardHeader.appendChild(titleContainer);

        const allControls = document.createElement('div');
        allControls.className = 'd-flex align-items-center';

        const btnGroup = document.createElement("div");
        btnGroup.className = "btn-group";
        btnGroup.setAttribute("role", "group");

        const prevButton = document.createElement("button");
        prevButton.className = "btn btn-sm btn-outline-secondary";
        prevButton.id = 'prev-btn-' + id;
        prevButton.innerHTML = '<i class="bi bi-skip-backward-fill"></i>';
        prevButton.addEventListener("click", () => stepFrame(id, -1));

        const pauseButton = document.createElement("button");
        pauseButton.className = "btn btn-sm btn-outline-secondary";
        pauseButton.id = 'pause-btn-' + id;
        pauseButton.innerHTML = '<i class="bi bi-play-fill"></i>';
        pauseButton.addEventListener("click", () => togglePause(id));

        const nextButton = document.createElement("button");
        nextButton.className = "btn btn-sm btn-outline-secondary";
        nextButton.id = 'next-btn-' + id;
        nextButton.innerHTML = '<i class="bi bi-skip-forward-fill"></i>';
        nextButton.addEventListener("click", () => stepFrame(id, 1));

        const replayButton = document.createElement("button");
        replayButton.className = "btn btn-sm btn-outline-primary ms-2";
        replayButton.innerHTML = '<i class="bi bi-arrow-clockwise"></i>';
        replayButton.addEventListener("click", () => replayAnimation(id));

        btnGroup.appendChild(prevButton);
        btnGroup.appendChild(pauseButton);
        btnGroup.appendChild(nextButton);

        allControls.appendChild(btnGroup);
        allControls.appendChild(replayButton);

        cardHeader.appendChild(allControls);
        wrapper.appendChild(cardHeader);

        const canvas = document.createElement("canvas");
        canvas.id = 'canvas-' + id;
        wrapper.appendChild(canvas);

        const parallelRulesWrapper = document.createElement("div");
        parallelRulesWrapper.className = "parallel-rules-wrapper";
        parallelRulesWrapper.id = 'parallel-rules-list-' + id;
        parallelRulesWrapper.innerHTML = '<span class="text-muted">Compatible Parallel Rules:</span> <span class="fw-bold">...</span>';
        wrapper.appendChild(parallelRulesWrapper);

        col.appendChild(wrapper);
        document.getElementById("canvasContainer").appendChild(col);
        return canvas.getContext("2d");
      }}

      function drawGrid(ctx, size, scale, offset) {{
        ctx.strokeStyle = '#e9ecef';
        ctx.lineWidth = 1;
        ctx.setLineDash([4, 4]);
        const gridRange = 5;
        for (let i = -gridRange; i <= gridRange; i++) {{
          if (i === 0) continue;
          ctx.beginPath();
          ctx.moveTo(offset + i * scale, 0);
          ctx.lineTo(offset + i * scale, size);
          ctx.stroke();
          ctx.beginPath();
          ctx.moveTo(0, offset - i * scale);
          ctx.lineTo(size, offset - i * scale);
          ctx.stroke();
        }}
        ctx.setLineDash([]);
      }}

      function drawFrame(ctx, points) {{
        const canvas = ctx.canvas;
        if (canvas.width !== canvas.clientWidth) {{
          canvas.width = canvas.clientWidth;
          canvas.height = canvas.clientWidth;
        }}
        const size = canvas.width;
        if (size === 0) return;

        const offset = size / 2;
        const scale = size / 10;

        ctx.clearRect(0, 0, size, size);
        drawGrid(ctx, size, scale, offset);

        ctx.strokeStyle = '#adb5bd';
        ctx.lineWidth = 2;
        ctx.beginPath();
        ctx.moveTo(offset, 0);
        ctx.lineTo(offset, size);
        ctx.moveTo(0, offset);
        ctx.lineTo(size, offset);
        ctx.stroke();

        points.forEach(([color, x, y]) => {{
          ctx.fillStyle = config.colors[color] || "black";
          ctx.beginPath();
          ctx.arc(offset + x * scale, offset - y * scale, size * 0.02, 0, Math.PI * 2);
          ctx.fill();
        }});
      }}

      function setupAndPlaySequences(play) {{
        document.getElementById("canvasContainer").innerHTML = "";
        Object.values(config.animations).forEach(anim => clearTimeout(anim.timeoutId));
        config.animations = {{}};
        config.data.forEach((sequence, index) => {{
          startAnimation(index, sequence, play);
        }});
        updateGlobalPauseResumeIcon(!play);
      }}

      function togglePauseResumeAll() {{
        const hasRunningAnimation = Object.values(config.animations).some(anim => anim && !anim.isPaused);

        if (hasRunningAnimation) {{
          Object.keys(config.animations).forEach(index => {{
            const anim = config.animations[index];
            if (anim && !anim.isPaused) {{
              anim.isPaused = true;
              clearTimeout(anim.timeoutId);
              updatePauseButtonIcon(parseInt(index), true);

              if (anim.currentFrame > 0) {{
                anim.currentFrame = anim.currentFrame - 1;
              }}

              if (anim.currentFrame >= anim.sequence.length) {{
                anim.currentFrame = anim.sequence.length - 1;
              }}
              if (anim.currentFrame < 0) {{
                anim.currentFrame = 0;
              }}

              updateButtonStates(parseInt(index));
              updateStepInfo(parseInt(index));
            }}
          }});
          updateGlobalPauseResumeIcon(true);
        }} else {{
          Object.keys(config.animations).forEach(index => {{
            const anim = config.animations[index];
            if (anim && anim.isPaused) {{
              anim.isPaused = false;
              updatePauseButtonIcon(parseInt(index), false);

              if (anim.currentFrame >= anim.sequence.length) {{
                anim.currentFrame = 0;
              }}
              animationStep(parseInt(index), true);
            }}
          }});
          updateGlobalPauseResumeIcon(false);
        }}
      }}

      function updateGlobalPauseResumeIcon(isPaused) {{
        const button = document.getElementById("pauseResumeAll");
        if (button) {{
          button.innerHTML = isPaused ? '<i class="bi bi-play-fill"></i>' : '<i class="bi bi-pause-fill"></i>';
          button.title = isPaused ? "Resume All" : "Pause All";
        }}
      }}

      function previousAllSteps() {{
        Object.keys(config.animations).forEach(index => {{
          const anim = config.animations[index];
          if (anim) {{
            if (!anim.isPaused) {{
              anim.isPaused = true;
              clearTimeout(anim.timeoutId);
              updatePauseButtonIcon(parseInt(index), true);
            }}

            if (anim.currentFrame >= anim.sequence.length) {{
              anim.currentFrame = anim.sequence.length - 1;
            }}

            if (anim.currentFrame > 0) {{
              anim.currentFrame = anim.currentFrame - 1;
              drawFrame(anim.ctx, anim.sequence[anim.currentFrame]);
              updateButtonStates(parseInt(index));
              updateStepInfo(parseInt(index));
            }}
          }}
        }});
      }}

      function nextAllSteps() {{
        Object.keys(config.animations).forEach(index => {{
          const anim = config.animations[index];
          if (anim) {{
            if (!anim.isPaused) {{
              anim.isPaused = true;
              clearTimeout(anim.timeoutId);
              updatePauseButtonIcon(parseInt(index), true);
            }}

            if (anim.currentFrame >= anim.sequence.length) {{
              anim.currentFrame = anim.sequence.length - 1;
            }}

            if (anim.currentFrame < anim.sequence.length - 1) {{
              anim.currentFrame = anim.currentFrame + 1;
              drawFrame(anim.ctx, anim.sequence[anim.currentFrame]);
              updateButtonStates(parseInt(index));
              updateStepInfo(parseInt(index));
            }}
          }}
        }});
      }}

      function startAnimation(index, sequence, play = false) {{
        const ctx = createCanvas(index);

        updateParallelRulesHighlight(index, 0);

        config.animations[index] = {{
          timeoutId: null,
          currentFrame: 0,
          isPaused: !play,
          sequence: sequence,
          ctx: ctx
        }};

        if (sequence && sequence.length > 0) {{
            drawFrame(ctx, sequence[0]);
            updateStepInfo(index);
        }}
        updatePauseButtonIcon(index, !play);
        updateButtonStates(index);

        if (play) {{
            animationStep(index, true);
        }}
      }}

      function animationStep(index, immediate = false) {{
          const anim = config.animations[index];
          if (!anim || anim.isPaused) return;

          if (anim.currentFrame < anim.sequence.length) {{
              drawFrame(anim.ctx, anim.sequence[anim.currentFrame]);
              updateButtonStates(index);
              updateStepInfo(index);

              anim.currentFrame++;

              const delay = immediate ? 0 : 1000;
              anim.timeoutId = setTimeout(() => animationStep(index, false), delay);
          }} else {{
              anim.isPaused = true;
              anim.currentFrame = 0;
              drawFrame(anim.ctx, anim.sequence[0]);
              updatePauseButtonIcon(index, true);
              updateButtonStates(index);
              updateStepInfo(index);
          }}
      }}

      function replayAnimation(index) {{
        const anim = config.animations[index];
        if (!anim) return;

        clearTimeout(anim.timeoutId);
        anim.currentFrame = 0;
        anim.isPaused = false;
        updatePauseButtonIcon(index, false);
        animationStep(index, true);
      }}

      function togglePause(index) {{
          const anim = config.animations[index];
          if (!anim) return;

          if (anim.isPaused) {{
              anim.isPaused = false;
              updatePauseButtonIcon(index, false);

              if (anim.currentFrame >= anim.sequence.length) {{
                  anim.currentFrame = 0;
              }}
              animationStep(index, true);
          }} else {{
              anim.isPaused = true;
              clearTimeout(anim.timeoutId);
              updatePauseButtonIcon(index, true);

              if (anim.currentFrame > 0) {{
                  anim.currentFrame = anim.currentFrame - 1;
              }}

              if (anim.currentFrame >= anim.sequence.length) {{
                  anim.currentFrame = anim.sequence.length - 1;
              }}
              if (anim.currentFrame < 0) {{
                  anim.currentFrame = 0;
              }}

              updateButtonStates(index);
              updateStepInfo(index);
          }}
      }}

      function stepFrame(index, direction) {{
          const anim = config.animations[index];
          if (!anim) return;

          if (!anim.isPaused) {{
              anim.isPaused = true;
              clearTimeout(anim.timeoutId);
              updatePauseButtonIcon(index, true);
          }}

          if (anim.currentFrame >= anim.sequence.length) {{
            anim.currentFrame = anim.sequence.length - 1;
          }}

          const newFrame = anim.currentFrame + direction;

          if (newFrame >= 0 && newFrame < anim.sequence.length) {{
              anim.currentFrame = newFrame;
              drawFrame(anim.ctx, anim.sequence[anim.currentFrame]);
              updateButtonStates(index);
              updateStepInfo(index);
          }}
      }}

      function updatePauseButtonIcon(index, isPaused) {{
          const button = document.getElementById('pause-btn-' + index);
          if (button) {{
              button.innerHTML = isPaused ? '<i class="bi bi-play-fill"></i>' : '<i class="bi bi-pause-fill"></i>';
          }}
      }}

      function updateButtonStates(index) {{
        const anim = config.animations[index];
        if (!anim) return;

        const prevButton = document.getElementById('prev-btn-' + index);
        const nextButton = document.getElementById('next-btn-' + index);

        if (!prevButton || !nextButton) return;

        prevButton.disabled = anim.currentFrame <= 0;
        nextButton.disabled = anim.currentFrame >= anim.sequence.length - 1;
      }}

      function updateStepInfo(index) {{
        const anim = config.animations[index];
        if (!anim) return;

        const stepInfo = document.getElementById('step-info-' + index);
        if (stepInfo) {{
          const currentStep = anim.currentFrame;
          const totalSteps = anim.sequence.length - 1;
          stepInfo.innerText = 'Step ' + currentStep + '/' + totalSteps;
        }}

        updateParallelRulesHighlight(index, anim.currentFrame);
      }}

      function updateParallelRulesHighlight(index, currentFrame) {{
        const parallelRulesListElement = document.getElementById('parallel-rules-list-' + index);
        if (!parallelRulesListElement) return;

        const parallelRulesData = config.executions[index] || [];
        if (parallelRulesData.length === 0) {{
          parallelRulesListElement.innerHTML = '<span class="text-muted">Compatible Parallel Rules:</span> <span class="fw-bold">N/A</span>';
          return;
        }}

        const highlightedIndex = currentFrame;
        const rulesHtml = parallelRulesData.map((rule, idx) => {{
          if (idx === highlightedIndex) {{
            return '<strong class="text-primary">' + rule + '</strong>';
          }} else {{
            return rule.toString();
          }}
        }}).join(', ');

        parallelRulesListElement.innerHTML = '<span class="text-muted">Compatible Parallel Rules:</span> <span class="fw-bold">' + rulesHtml + '</span>';
      }}
    </script>
  </body>
</html>"#,
        goal_index, target_index, goal_index, target_index, positions_json, executions_json
    )
}
