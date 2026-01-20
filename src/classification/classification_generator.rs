use fxhash::FxHasher;
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs;
use std::hash::{Hash, Hasher};

use crate::{
    classification::logic::group_by_hash,
    modules::algorithm_experiments_modules::algorithm_experiments::AlgorithmExperiments,
    modules::classification::{
        algorithm_info::AlgorithmInfo, classification_result::ClassificationResult,
        family_category::FamilyCategory, family_group::FamilyGroup, family_summary::FamilySummary,
    },
};

pub fn export_classification(
    experiments: &[AlgorithmExperiments],
    output_path: &str,
    root_name: &str,
) {
    let mut families = Vec::new();

    // ===== GLOBAL METRICS (Families 1-3) =====
    families.push(create_family_category(
        experiments,
        1,
        "Number of Rules",
        "Classification based on the total number of rules in the algorithm",
        |a| a.hash_by_rules_count(),
    ));

    families.push(create_family_category(
        experiments,
        2,
        "Number of Rounds in a Cycle",
        "Classification based on the number of rounds within cycles across executions",
        |a| a.hash_by_cycle_len_in_executions(),
    ));

    families.push(create_family_category(
        experiments,
        3,
        "Energy in a Cycle",
        "Classification based on the total activation energy within cycles across executions",
        |a| a.hash_by_total_activation_in_cycle_in_executions(),
    ));
    families.push(create_family_category(
        experiments,
        4,
        "Paths in a Cycle",
        "Classification based on the paths robots take within cycles across executions",
        |a| a.hash_by_cycle_paths_in_executions(),
    ));

    families.push(create_family_category(
        experiments,
        5,
        "Number of rules without movement",
        "Classification based on the number of idle rules in the algorithm",
        |a| a.hash_by_idle_rules_count(),
    ));

    families.push(create_family_category(
        experiments,
        6,
        "Number of rules with invisible zone",
        "Classification based on the number of opacity rules in the algorithm",
        |a| a.hash_by_opac_rules_count(),
    ));

    // ===== BY ROBOT COLOR (Families 4-6) =====
    families.push(create_family_category(
        experiments,
        7,
        "Number of rules for each color",
        "Classification based on the number of rules per robot color",
        |a| a.hash_family_by_rules_count_by_colors(),
    ));

    families.push(create_family_category(
        experiments,
        8,
        "Number of rules without movement for each color",
        "Classification based on the number of idle rules per robot color",
        |a| a.hash_family_by_idle_rules_count_by_colors(),
    ));

    families.push(create_family_category(
        experiments,
        9,
        "Number of rules with invisible zone for each color",
        "Classification based on the number of opacity rules per robot color",
        |a| a.hash_family_by_opacity_rules_count_by_colors(),
    ));

    // ===== EXECUTION PATTERNS - TOTAL (Families 7-11) =====
    families.push(create_family_category(
        experiments,
        10,
        "Number of rules used in an execution",
        "Classification based on the number of rules used across executions",
        |a| a.hash_by_rules_count_in_executions(),
    ));

    families.push(create_family_category(
        experiments,
        11,
        "Energy in an execution",
        "Classification based on the total energy consumed across executions",
        |a| a.hash_by_total_activation_in_executions(),
    ));

    families.push(create_family_category(
        experiments,
        12,
        "Number of color changes in an execution",
        "Classification based on the energy consumed for color changes across executions",
        |a| a.hash_by_color_activation_in_executions(),
    ));

    families.push(create_family_category(
        experiments,
        13,
        "Number of movements in an execution",
        "Classification based on the energy consumed for movements across executions",
        |a| a.hash_by_movement_activation_in_executions(),
    ));

    families.push(create_family_category(
        experiments,
        14,
        "Number of rounds in an execution",
        "Classification based on the total number of rounds taken across executions",
        |a| a.hash_by_steps_taken_in_executions(),
    ));

    // ===== EXECUTION PATTERNS - BY ROBOT (Families 13-16) =====
    families.push(create_family_category(
        experiments,
        15,
        "Number of rules used in an execution for each robot",
        "Classification based on the number of rules used per robot across executions",
        |a| a.hash_by_rules_count_in_executions_by_robot(),
    ));

    families.push(create_family_category(
        experiments,
        16,
        "Energy in an execution for each robot",
        "Classification based on the total energy consumed per robot across executions",
        |a| a.hash_by_activation_in_executions_by_robot(),
    ));

    families.push(create_family_category(
        experiments,
        17,
        "Number of color changes in an execution for each robot",
        "Classification based on the energy consumed for color changes per robot across executions",
        |a| a.hash_by_color_activation_in_executions_by_robot(),
    ));

    families.push(create_family_category(
        experiments,
        18,
        "Number of movements in an execution for each robot",
        "Classification based on the energy consumed for movements per robot across executions",
        |a| a.hash_by_movement_activation_in_executions_by_robot(),
    ));

    // ===== FINAL POSITIONS AND PATHS (Families 17-19) =====
    families.push(create_family_category(
        experiments,
        19,
        "Sequence of colors in an execution",
        "Classification based on the color changes per robot across executions",
        |a| a.hash_by_color_in_executions(),
    ));

    families.push(create_family_category(
        experiments,
        20,
        "Sequence of configurations in an execution",
        "Classification based on the paths taken by each robot across executions",
        |a| a.hash_by_paths_in_executions(),
    ));

    families.push(create_family_category(
        experiments,
        21,
        "Paths in an execution",
        "Classification based on the positions occupied by each robot across executions",
        |a| a.hash_by_positions_in_executions(),
    ));

    families.push(create_family_category(
        experiments,
        22,
        "Energy in the execution prefix before the cycle",
        "Classification based on the energy consumed before entering a cycle across executions",
        |a| a.hash_by_activation_in_executions_before_cycle(),
    ));

    families.push(create_family_category(
        experiments,
        23,
        "Paths in the execution prefix before the cycle",
        "Classification based on the paths taken before entering a cycle across executions",
        |a| a.hash_by_paths_before_cycle_in_executions(),
    ));

    // Generate summaries
    let summaries: Vec<FamilySummary> = families
        .iter()
        .map(|family| {
            let total_algos: usize = family.groups.iter().map(|g| g.algorithm_count).sum();
            let group_counts: Vec<String> = family
                .groups
                .iter()
                .map(|g| g.algorithm_count.to_string())
                .collect();
            let details = format!("{}={}", total_algos, group_counts.join("+"));

            FamilySummary {
                id: family.family_number,
                family_name: family.title.clone(),
                total_groups: family.groups.len(),
                details,
            }
        })
        .collect();

    let result = ClassificationResult {
        total_algorithms: experiments.len(),
        classification_date: chrono::Utc::now().to_rfc3339(),
        families,
        summaries,
    };

    // Generate HTML with embedded JSON
    match generate_classification_html(
        &result,
        format!("{}/{}classification.html", output_path, root_name).as_str(),
    ) {
        Ok(_) => {}
        Err(e) => eprintln!("✗ Failed to generate HTML: {}", e),
    }
}

fn create_family_category<F>(
    experiments: &[AlgorithmExperiments],
    family_num: usize,
    title: &str,
    description: &str,
    hash_fn: F,
) -> FamilyCategory
where
    F: Fn(&AlgorithmExperiments) -> String,
{
    let families = group_by_hash(experiments, |a| hash_fn(a));

    let mut groups: Vec<FamilyGroup> = families
        .into_iter()
        .map(|(hash_string, algos)| {
            /*    // Use original string if short (<= 20 chars), otherwise compute compact hash
                        let display_hash = if hash_string.len() <= 20 {
                            hash_string.clone()
                        } else {
                            // Compute a compact 64-bit hash using FxHasher (same as remove_duplicates_indexed_simple_fast)
                            let mut hasher = FxHasher::default();
                            hash_string.hash(&mut hasher);
                            let hash_u64 = hasher.finish();
                            format!("{:016x}", hash_u64) // Format as 16-character hex string
                        };
            */
            // Always compute compact hash using FxHasher
            let mut hasher = FxHasher::default();
            hash_string.hash(&mut hasher);
            let hash_u64 = hasher.finish();
            let display_hash = format!("{:016x}", hash_u64); // Format as 16-character hex string

            // Store the original signature for display
            let signature = if hash_string.len() > 10000 {
                "Too long!!".to_string()
            } else {
                hash_string.clone()
            };

            FamilyGroup {
                hash: display_hash,
                signature,
                algorithm_count: algos.len(),
                algorithms: algos
                    .iter()
                    .map(|a| {
                        let properties = build_properties_for_family(a, family_num);

                        AlgorithmInfo {
                            name: a.name.clone(),
                            properties,
                        }
                    })
                    .collect(),
            }
        })
        .collect();

    groups.sort_by(|a, b| a.hash.cmp(&b.hash));

    FamilyCategory {
        family_number: family_num,
        title: title.to_string(),
        description: description.to_string(),
        total_families: groups.len(),
        groups,
    }
}

fn build_properties_for_family(
    algo: &AlgorithmExperiments,
    family_num: usize,
) -> Vec<(String, String)> {
    let total_rules: usize = algo
        .infos
        .by_robot_colors
        .iter()
        .map(|info| info.rules_count)
        .sum();
    let total_idle: usize = algo
        .infos
        .by_robot_colors
        .iter()
        .map(|info| info.idle_rules_count)
        .sum();
    let total_opacity: usize = algo
        .infos
        .by_robot_colors
        .iter()
        .map(|info| info.opacity_rule_count)
        .sum();

    let mut properties = Vec::new();

    match family_num {
        _ => {
            // Default: show all info
            // Standard info shown for every algorithm (regardless of family)
            properties.push(("Total Rules".to_string(), total_rules.to_string()));
            properties.push(("Status".to_string(), format!("{:?}", algo.status)));
            properties.push((
                "Experiments".to_string(),
                algo.experiments.len().to_string(),
            ));
            let total_color_activations_in_all: usize = algo
                .experiments
                .iter()
                .map(|exp| {
                    exp.robots_metrics
                        .iter()
                        .map(|rm| rm.color_activations)
                        .sum::<usize>()
                })
                .sum();
            let total_movement_activations_in_all: usize = algo
                .experiments
                .iter()
                .map(|exp| {
                    exp.robots_metrics
                        .iter()
                        .map(|rm| rm.movement_activations)
                        .sum::<usize>()
                })
                .sum();
            let total_activations_in_all: usize =
                total_color_activations_in_all + total_movement_activations_in_all;
            let total_steps_in_all: usize =
                algo.experiments.iter().map(|exp| exp.steps_taken).sum();
            let total_rules_in_all: usize = algo
                .experiments
                .iter()
                .map(|exp| {
                    exp.robots_metrics
                        .iter()
                        .map(|rm| rm.rule_count)
                        .sum::<usize>()
                })
                .sum();

            properties.push((
                "Total Energy in All Experiments".to_string(),
                total_activations_in_all.to_string(),
            ));
            properties.push((
                "Total Color Energy in All Experiments".to_string(),
                total_color_activations_in_all.to_string(),
            ));
            properties.push((
                "Total Movement Energy in All Experiments".to_string(),
                total_movement_activations_in_all.to_string(),
            ));
            properties.push((
                "Total Rounds Taken in All Experiments".to_string(),
                total_steps_in_all.to_string(),
            ));

            properties.push((
                "Total Rules in All Experiments".to_string(),
                total_rules_in_all.to_string(),
            ));
        }
    }

    properties
}

pub fn generate_classification_html(
    data: &ClassificationResult,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let json_data = serde_json::to_string(data)?;

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Algorithm Classification Dashboard</title>
    <link href="https://cdn.jsdelivr.net/npm/bootstrap@5.3.2/dist/css/bootstrap.min.css" rel="stylesheet">
    <link href="https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600;700&display=swap" rel="stylesheet">
    <style>
        :root {{
            --primary-color: #4f46e5;
            --primary-hover: #4338ca;
            --secondary-color: #64748b;
            --bg-color: #f1f5f9;
            --card-bg: #ffffff;
            --text-primary: #1e293b;
            --text-secondary: #64748b;
            --border-color: #e2e8f0;
            --success-color: #10b981;
            --warning-color: #f59e0b;
        }}

        body {{
            background: var(--bg-color);
            font-family: 'Inter', sans-serif;
            color: var(--text-primary);
            padding-bottom: 40px;
        }}

        .container {{
            max-width: 1400px;
            margin: 0 auto;
            padding: 0 20px;
        }}

        .header {{
            background: linear-gradient(135deg, #ffffff 0%, #f8fafc 100%);
            padding: 25px;
            margin-bottom: 25px;
            border-radius: 12px;
            border: 1px solid var(--border-color);
            box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.05);
        }}

        .header h1 {{
            font-size: 1.8rem;
            font-weight: 700;
            margin: 0 0 10px 0;
            color: var(--text-primary);
            letter-spacing: -0.025em;
        }}

        .stats {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 15px;
            margin-top: 20px;
        }}

        .stat {{
            background: white;
            padding: 15px;
            border-radius: 8px;
            border: 1px solid var(--border-color);
            box-shadow: 0 1px 3px rgba(0,0,0,0.05);
            transition: transform 0.2s ease;
        }}

        .stat:hover {{
            transform: translateY(-2px);
        }}

        .stat-value {{
            font-size: 1.5rem;
            font-weight: 700;
            color: var(--primary-color);
            line-height: 1.2;
        }}

        .stat-label {{
            font-size: 0.85rem;
            color: var(--text-secondary);
            font-weight: 500;
            margin-top: 4px;
            text-transform: uppercase;
            letter-spacing: 0.05em;
        }}

        .control-panel {{
            background: var(--card-bg);
            padding: 20px;
            margin-bottom: 25px;
            border-radius: 12px;
            border: 1px solid var(--border-color);
            box-shadow: 0 1px 3px rgba(0,0,0,0.05);
        }}

        .control-panel label {{
            font-weight: 600;
            margin-bottom: 8px;
            display: block;
            color: var(--text-primary);
            font-size: 0.95rem;
        }}

        .family-dropdown {{
            width: 100%;
            padding: 10px 15px;
            border: 1px solid var(--border-color);
            border-radius: 8px;
            font-size: 1rem;
            background-color: #fff;
            transition: border-color 0.2s, box-shadow 0.2s;
        }}

        .family-dropdown:focus {{
            outline: none;
            border-color: var(--primary-color);
            box-shadow: 0 0 0 3px rgba(79, 70, 229, 0.1);
        }}

        .content-area {{
            background: transparent;
        }}

        .family-info {{
            background: var(--card-bg);
            padding: 20px;
            border-radius: 12px;
            border: 1px solid var(--border-color);
            margin-bottom: 25px;
            border-left: 5px solid var(--primary-color);
            box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.05);
        }}

        .family-info h2 {{
            font-size: 1.5rem;
            color: var(--text-primary);
            margin: 0 0 8px 0;
            font-weight: 700;
        }}

        .family-info .description {{
            color: var(--text-secondary);
            margin: 0;
            font-size: 1rem;
            line-height: 1.5;
        }}

        .group-card {{
            background: var(--card-bg);
            border: 1px solid var(--border-color);
            border-radius: 12px;
            padding: 20px;
            margin-bottom: 25px;
            box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.05);
        }}

        .group-header {{
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 20px;
            padding-bottom: 15px;
            border-bottom: 1px solid var(--border-color);
        }}

        .group-hash-container {{
            flex: 1;
            margin-right: 15px;
        }}

        .group-hash {{
            font-size: 1rem;
            font-weight: 600;
            color: var(--text-primary);
            font-family: 'Monaco', 'Consolas', monospace;
            background: #f1f5f9;
            padding: 4px 8px;
            border-radius: 4px;
            display: inline-block;
        }}

        .group-count {{
            background: var(--primary-color);
            color: white;
            padding: 6px 14px;
            border-radius: 20px;
            font-size: 0.85rem;
            font-weight: 600;
            white-space: nowrap;
            box-shadow: 0 2px 4px rgba(79, 70, 229, 0.2);
        }}

        .algorithms-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
            gap: 20px;
        }}

        .algo-card {{
            background: #fff;
            border: 1px solid var(--border-color);
            border-radius: 8px;
            padding: 15px;
            transition: all 0.2s ease;
        }}

        .algo-card:hover {{
            transform: translateY(-3px);
            box-shadow: 0 10px 15px -3px rgba(0, 0, 0, 0.1);
            border-color: var(--primary-color);
        }}

        .algo-header {{
            display: flex;
            justify-content: space-between;
            align-items: center;
            transition: all 0.2s ease;
            cursor: pointer;
            border-radius: 6px;
            padding: 8px;
            margin: -8px -8px 0 -8px; 
        }}

        .algo-header:hover {{
            background-color: #f8fafc;
        }}

        .algo-header.expanded {{
            background-color: #f1f5f9;
            margin-bottom: 12px;
            border-bottom: 1px solid #e2e8f0;
            border-radius: 6px 6px 0 0;
        }}

        .family-header-container {{
            display: flex;
            justify-content: space-between;
            align-items: flex-start;
        }}

        .family-toggle-btn {{
            background: #f1f5f9;
            border: none;
            color: var(--text-secondary);
            border-radius: 6px;
            padding: 8px;
            cursor: pointer;
            transition: all 0.2s;
            display: flex;
            align-items: center;
            justify-content: center;
        }}

        .family-toggle-btn:hover {{
            background: #e2e8f0;
            color: var(--primary-color);
        }}
        
        .family-toggle-btn.expanded svg {{
            transform: rotate(180deg);
        }}
        
        .family-toggle-btn svg {{
            transition: transform 0.3s ease;
        }}

        .algo-name {{
            font-weight: 700;
            color: var(--text-primary);
            font-size: 1rem;
            margin: 0;
            line-height: 1.4;
            word-break: break-all;
            display: flex;
            align-items: center;
            gap: 10px;
        }}

        .toggle-btn {{
            background: none;
            border: none;
            color: var(--text-secondary);
            cursor: pointer;
            padding: 4px;
            border-radius: 4px;
            transition: all 0.2s ease;
            display: flex;
            align-items: center;
            justify-content: center;
            margin-left: 10px;
            flex-shrink: 0;
        }}

        .toggle-btn:hover {{
            background-color: #f1f5f9;
            color: var(--primary-color);
        }}

        .toggle-btn svg {{
            transition: transform 0.3s ease;
        }}
        
        .toggle-btn.expanded svg {{
            transform: rotate(180deg);
        }}

        .algo-stats {{
            display: flex;
            flex-direction: column;
            gap: 8px;
        }}

        .stat-item {{
            display: flex;
            justify-content: space-between;
            align-items: center;
            font-size: 0.85rem;
        }}

        .stat-item .stat-label {{
            color: var(--text-secondary);
            margin: 0;
            text-transform: none;
            letter-spacing: normal;
            font-weight: 400;
        }}

        .stat-item .stat-value {{
            font-weight: 600;
            color: var(--text-primary);
            font-size: 0.9rem;
        }}

        .loading {{
            text-align: center;
            padding: 40px;
            color: var(--text-secondary);
            font-size: 1rem;
        }}

        .empty-state {{
            text-align: center;
            padding: 60px 20px;
            color: var(--text-secondary);
            background: white;
            border-radius: 12px;
            border: 1px dashed var(--border-color);
        }}

        .empty-state h3 {{
            font-size: 1.5rem;
            margin-bottom: 10px;
            color: var(--text-primary);
        }}

        .empty-state p {{
            font-size: 1rem;
            max-width: 500px;
            margin: 0 auto;
        }}

        @media (max-width: 768px) {{
            .algorithms-grid {{
                grid-template-columns: 1fr;
            }}
            .stats {{
                grid-template-columns: 1fr;
            }}
            .group-header {{
                flex-direction: column;
                align-items: flex-start;
                gap: 10px;
            }}
            .group-count {{
                align-self: flex-start;
            }}
        }}

        .summary-section {{
            margin-top: 30px;
            background: white;
            border-radius: 12px;
            padding: 20px;
            border: 1px solid var(--border-color);
            box-shadow: 0 1px 3px rgba(0,0,0,0.05);
        }}
        
        .summary-section h5 {{
            font-weight: 700;
            color: var(--text-primary);
            margin-bottom: 15px;
            padding-bottom: 10px;
            border-bottom: 1px solid var(--border-color);
        }}

        .table {{
            margin-bottom: 0;
        }}

        .table thead th {{
            background-color: #f8fafc;
            color: var(--text-secondary);
            font-weight: 600;
            text-transform: uppercase;
            font-size: 0.75rem;
            letter-spacing: 0.05em;
            border-bottom: 2px solid var(--border-color);
        }}

        .table td {{
            vertical-align: middle;
            color: var(--text-primary);
            font-size: 0.9rem;
        }}

        .summary-section table tbody tr {{
            cursor: pointer;
            transition: background-color 0.2s;
        }}

        .summary-section table tbody tr:hover {{
            background-color: #eef2ff !important;
        }}

        .summary-section table tbody tr:hover td {{
            color: var(--primary-color);
            font-weight: 500;
        }}

        /* Modal styles */
        .modal {{
            display: none;
            position: fixed;
            z-index: 1000;
            left: 0;
            top: 0;
            width: 100%;
            height: 100%;
            background-color: rgba(0, 0, 0, 0.5);
        }}

        .modal-content {{
            background-color: white;
            margin: 5% auto;
            padding: 0;
            border: 1px solid var(--border-color);
            border-radius: 12px;
            width: 80%;
            max-width: 800px;
            max-height: 80vh;
            display: flex;
            flex-direction: column;
            box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.1);
        }}

        .modal-header {{
            padding: 20px;
            border-bottom: 1px solid var(--border-color);
            display: flex;
            justify-content: space-between;
            align-items: center;
        }}

        .modal-header h3 {{
            margin: 0;
            color: var(--text-primary);
            font-size: 1.25rem;
        }}

        .close {{
            font-size: 28px;
            font-weight: bold;
            color: var(--text-secondary);
            cursor: pointer;
            background: none;
            border: none;
            padding: 0;
            line-height: 1;
        }}

        .close:hover {{
            color: var(--text-primary);
        }}

        .modal-body {{
            padding: 20px;
            overflow-y: auto;
            flex: 1;
        }}

        .signature-textarea {{
            width: 100%;
            min-height: 300px;
            padding: 15px;
            font-family: 'Monaco', 'Consolas', monospace;
            font-size: 13px;
            border: 1px solid var(--border-color);
            border-radius: 8px;
            background: #f8fafc;
            color: var(--text-primary);
            resize: vertical;
        }}

        .signature-btn {{
            background: var(--primary-color);
            color: white;
            border: none;
            padding: 6px 12px;
            border-radius: 6px;
            font-size: 0.85rem;
            cursor: pointer;
            margin-left: 10px;
            transition: background-color 0.2s;
        }}

        .signature-btn:hover {{
            background: var(--primary-hover);
        }}


    </style>
</head>
<body>
    <div class="container">
        <!-- Header -->
        <div class="header">
            <h1>Algorithm Classification</h1>
            <p style="color: #666; margin: 0;">Explore algorithm families based on various metrics</p>
            
            <div class="stats">
                <div class="stat">
                    <div class="stat-value" id="totalAlgorithms">-</div>
                    <div class="stat-label">Total Algorithms</div>
                </div>
                <div class="stat">
                    <div class="stat-value" id="classificationDate">-</div>
                    <div class="stat-label">Classification Date</div>
                </div>
                <div class="stat" id="groupCountCard" style="display: none;">
                    <div class="stat-value" id="groupCount">0</div>
                    <div class="stat-label">Groups</div>
                </div>
            </div>

            <!-- Summary Table -->
            <div class="summary-section">
                <h5>Family Summaries</h5>
                <div class="table-responsive">
                    <table class="table table-hover">
                        <thead>
                            <tr>
                                <th style="width: 60px; text-align: center;">ID</th>
                                <th style="min-width: 280px; white-space: nowrap;">Family Name</th>
                                <th style="width: 120px; text-align: center;">Groups</th>
                                <th>Breakdown (Total = Sum of Groups)</th>
                            </tr>
                        </thead>
                        <tbody id="summaryTableBody">
                            <!-- Populated by JS -->
                        </tbody>
                    </table>
                </div>
            </div>
        </div>

        <!-- Control Panel -->
        <div class="control-panel">
            <label for="familySelect">Select Family Category</label>
            <select class="family-dropdown" id="familySelect">
                <option value="">-- Choose a classification family --</option>
            </select>
        </div>

        <!-- Content Area -->
        <div class="content-area">
            <div id="emptyState" class="empty-state">
                <h3>Select a Family Category</h3>
                <p>Choose a classification family from the dropdown above to view the grouped algorithms.</p>
            </div>

            <div id="contentDisplay" style="display: none;">
                <div class="family-info" id="familyInfo"></div>
                <div id="groupsContainer"></div>
            </div>
        </div>
    </div>

    <!-- Signature Modal -->
    <div id="signatureModal" class="modal">
        <div class="modal-content">
            <div class="modal-header">
                <h3>Signature Details</h3>
                <button class="close" onclick="closeSignatureModal()">&times;</button>
            </div>
            <div class="modal-body">
                <textarea id="signatureText" class="signature-textarea" readonly></textarea>
            </div>
        </div>
    </div>

    <script>
        // ====================================
        // CONFIGURATION: Set this to true to show only main families, false to show all families
        // ====================================
        const showOnlyMainFamilies = true;
        
        // Embedded JSON data
        const classificationData = {json_data};
        const mainFamiliesCount = {main_families_count};

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

        function initializeDashboard() {{
            // Update header stats
            document.getElementById('totalAlgorithms').textContent = classificationData.total_algorithms;
            const date = new Date(classificationData.classification_date);
            document.getElementById('classificationDate').textContent = date.toLocaleDateString();

            // Populate summary table
            const summaryBody = document.getElementById('summaryTableBody');
            classificationData.summaries.forEach(summary => {{
                // Skip other families if showOnlyMainFamilies is true
                if (showOnlyMainFamilies && summary.id > mainFamiliesCount) {{
                    return;
                }}
                
                // Add separator after main families
                if (summary.id === mainFamiliesCount + 1) {{
                    const separatorRow = document.createElement('tr');
                    separatorRow.innerHTML = `
                        <td colspan="4" style="background: #f8f9fa; padding: 12px; text-align: center; font-weight: 700; color: #6c757d; border-top: 2px solid #dee2e6; border-bottom: 2px solid #dee2e6;">
                            Other Families
                        </td>
                    `;
                    summaryBody.appendChild(separatorRow);
                }}

                const row = document.createElement('tr');
                
                let statusIcon = "";
                if (summary.total_groups === 1) {{
                    statusIcon = '<span class="badge bg-success ms-2" title="All the algorithms are equivalent from the perspective of this family.">=</span>';
                }} else if (summary.total_groups === classificationData.total_algorithms) {{
                    statusIcon = '<span class="badge bg-danger ms-2" title="All the algorithms are different from the family’s perspective.">≠</span>';
                }}

                row.onclick = () => selectFamily(summary.id);
                row.title = "Click to view family details";
                row.innerHTML = `
                    <td class="text-center" style="font-weight: 600;">${{summary.id}}</td>
                    <td style="font-weight: 500; white-space: nowrap;">${{summary.family_name}} ${{statusIcon}}</td>
                    <td class="text-center"><span class="badge bg-secondary rounded-pill">${{summary.total_groups}}</span></td>
                    <td><code style="color: #d63384; background: #fdf2f8; padding: 2px 6px; border-radius: 4px;">${{summary.details}}</code></td>
                `;
                summaryBody.appendChild(row);
            }});

            // Populate family dropdown
            const familySelect = document.getElementById('familySelect');
            classificationData.families.forEach(family => {{
                // Skip other families if showOnlyMainFamilies is true
                if (showOnlyMainFamilies && family.family_number > mainFamiliesCount) {{
                    return;
                }}
                
                const option = document.createElement('option');
                option.value = family.family_number;
                option.textContent = `Family ${{family.family_number}}: ${{family.title}} (${{family.total_families}} group${{family.total_families !== 1 ? 's' : ''}})`;
                familySelect.appendChild(option);
            }});

            // Add event listener
            familySelect.addEventListener('change', handleFamilyChange);
        }}

        function handleFamilyChange(event) {{
            const familyNumber = parseInt(event.target.value);
            
            if (!familyNumber) {{
                document.getElementById('emptyState').style.display = 'block';
                document.getElementById('contentDisplay').style.display = 'none';
                document.getElementById('groupCount').textContent = '0';
                document.getElementById('groupCountCard').style.display = 'none';
                return;
            }}

            const family = classificationData.families.find(f => f.family_number === familyNumber);
            displayFamily(family);
        }}

        function displayFamily(family) {{
            // Hide empty state, show content
            document.getElementById('emptyState').style.display = 'none';
            document.getElementById('contentDisplay').style.display = 'block';

            // Update group count and show the card
            document.getElementById('groupCount').textContent = family.total_families;
            document.getElementById('groupCountCard').style.display = 'block';

            // Display family info
            // Display family info
            const familyInfoHTML = `
                <div class="family-header-container">
                    <div>
                        <h2>Family ${{family.family_number}}: ${{family.title}}</h2>
                        <p class="description">${{family.description}}</p>
                    </div>
                    <button class="family-toggle-btn" onclick="toggleAllFamily(this)" title="Toggle all algorithms">
                        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="6 9 12 15 18 9"></polyline></svg>
                    </button>
                </div>
            `;
            document.getElementById('familyInfo').innerHTML = familyInfoHTML;

            // Display groups
            displayGroups(family.groups);
        }}

        function displayGroups(groups) {{
            const container = document.getElementById('groupsContainer');
            
            if (groups.length === 0) {{
                container.innerHTML = '<div class="alert alert-info">No groups found in this family.</div>';
                return;
            }}

            let html = '';
            groups.forEach((group, index) => {{
                // Sort algorithms in each group using natural sort
                const sortedAlgorithms = [...group.algorithms].sort((a, b) => naturalSort(a.name, b.name));
                
                html += `
                    <div class="group-card">
                        <div class="group-header">
                            <div class="group-hash-container">
                                <span style="font-weight: 700; color: var(--primary-color); font-size: 1.1rem; margin-right: 12px;">Group ${{index + 1}}</span>
                                <div class="group-hash">${{escapeHtml(group.hash)}}</div>
                                <button class="signature-btn" onclick="showSignature('${{escapeHtml(group.signature).replace(/'/g, "\\'")}}')" title="View signature details">
                                    Details
                                </button>
                            </div>
                            <div class="group-count">${{group.algorithm_count}} Algorithm${{group.algorithm_count > 1 ? 's' : ''}}</div>
                        </div>
                        <div class="algorithms-grid">
                            ${{sortedAlgorithms.map(algo => createAlgorithmCard(algo)).join('')}}
                        </div>
                    </div>
                `;
            }});

            container.innerHTML = html;
        }}

        function toggleAllFamily(btn) {{
            const cards = document.querySelectorAll('.algo-card');
            
            // Check if any card is collapsed
            let hasCollapsed = false;
            cards.forEach(card => {{
                if (card.querySelector('.algo-stats').style.display === 'none') {{
                    hasCollapsed = true;
                }}
            }});

            const shouldExpand = hasCollapsed;

            if (shouldExpand) {{
                btn.classList.add('expanded');
            }} else {{
                btn.classList.remove('expanded');
            }}

            cards.forEach(card => {{
                const header = card.querySelector('.algo-header');
                const stats = card.querySelector('.algo-stats');
                
                if (shouldExpand) {{
                    if (stats.style.display === 'none') {{
                        stats.style.display = 'flex';
                        header.classList.add('expanded');
                    }}
                }} else {{
                    if (stats.style.display !== 'none') {{
                        stats.style.display = 'none';
                        header.classList.remove('expanded');
                    }}
                }}
            }});
        }}

        function toggleStats(element) {{
            const card = element.closest('.algo-card');
            const header = card.querySelector('.algo-header');
            const stats = card.querySelector('.algo-stats');
            
            if (stats.style.display === 'none') {{
                stats.style.display = 'flex';
                header.classList.add('expanded');
            }} else {{
                stats.style.display = 'none';
                header.classList.remove('expanded');
            }}
        }}

        function createAlgorithmCard(algo) {{
            const propertiesHTML = algo.properties.map(prop => `
                <div class="stat-item">
                    <span class="stat-label">${{escapeHtml(prop[0])}}</span>
                    <span class="stat-value">${{escapeHtml(prop[1])}}</span>
                </div>
            `).join('');
            
            // Icon for the algorithm (using a simple distinct shape like a hexagon or cube)
            const iconSvg = `
                <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke='#4f46e5' stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5"></path>
                </svg>
            `;

            return `
                <div class="algo-card">
                    <div class="algo-header" onclick="toggleStats(this)" title="Click to expanded details">
                        <div class="algo-name">
                            ${{iconSvg}}
                            <span>${{escapeHtml(algo.name)}}</span>
                        </div>
                    </div>
                    <div class="algo-stats" style="display: none;">
                        ${{propertiesHTML}}
                    </div>
                </div>
            `;
        }}

        function selectFamily(familyId) {{
            const select = document.getElementById('familySelect');
            select.value = familyId;
            
            // Trigger change event manually
            const event = new Event('change');
            select.dispatchEvent(event);
            
            // Scroll to control panel for better UX
            document.querySelector('.control-panel').scrollIntoView({{ behavior: 'smooth' }});
        }}

        function escapeHtml(text) {{
            const div = document.createElement('div');
            div.textContent = text;
            return div.innerHTML;
        }}

        function showSignature(signature) {{
            document.getElementById('signatureText').value = signature;
            document.getElementById('signatureModal').style.display = 'block';
        }}

        function closeSignatureModal() {{
            document.getElementById('signatureModal').style.display = 'none';
        }}

        // Close modal when clicking outside
        window.onclick = function(event) {{
            const modal = document.getElementById('signatureModal');
            if (event.target === modal) {{
                closeSignatureModal();
            }}
        }}

        // Close modal with Escape key
        window.onkeydown = function(event) {{
            if (event.key === 'Escape') {{
                closeSignatureModal();
            }}
        }}

        // Initialize on page load
        window.addEventListener('DOMContentLoaded', initializeDashboard);
    </script>
</body>
</html>"#,
        json_data = json_data,
        main_families_count = 4
    );

    fs::write(output_path, html)?;
    println!("✅ Classification HTML generated at: {}", output_path);

    Ok(())
}
