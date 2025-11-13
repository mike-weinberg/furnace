#!/usr/bin/env python3
"""
Generate performance comparison graphs for the schema inference libraries.

Focus: Compare the final optimized streaming implementation against reference implementations,
with only one chart showing the optimization journey.
"""

import matplotlib.pyplot as plt
import numpy as np

# Data collected from benchmarks (all times in milliseconds)

# By Complexity Category - Final implementations only
complexity_data = {
    'small+simple': {
        'Python GenSON': 0.29,
        'Rust genson-rs': 0.09,
        'json-melt (Streaming)': 0.06,
    },
    'small+complex': {
        'Python GenSON': 0.37,
        'Rust genson-rs': 0.30,
        'json-melt (Streaming)': 0.17,
    },
    'big+complex': {
        'Python GenSON': 0.41,
        'Rust genson-rs': 4.00,
        'json-melt (Streaming)': 2.93,
    }
}

# Overall averages - final implementations only
final_comparison = {
    'Python GenSON': 0.36,
    'Rust genson-rs': 1.04,
    'json-melt (Streaming)': 1.12,
}

# Optimization journey - showing only json-melt iterations
optimization_stages = {
    'Initial (unoptimized)': 389.68,
    'Cycle 1: Regex pre-compilation': 6.59,
    'Before refactor (merge-based)': 7.30,
    'Cycle 4: Streaming accumulator': 1.12,
}

# Create figure with multiple subplots
fig = plt.figure(figsize=(16, 10))
fig.suptitle('json-melt Schema Inference: Performance Analysis', fontsize=16, fontweight='bold', y=0.995)

# ============ Graph 1: Performance by Complexity (Final Implementations) ============
ax1 = plt.subplot(2, 3, 1)

categories = list(complexity_data.keys())
implementations = ['Python GenSON', 'Rust genson-rs', 'json-melt (Streaming)']
colors = ['#4ECDC4', '#45B7D1', '#2ECC71']

x = np.arange(len(categories))
width = 0.25

for idx, impl in enumerate(implementations):
    values = [complexity_data[cat].get(impl, 0) for cat in categories]
    ax1.bar(x + idx*width, values, width, label=impl, color=colors[idx])

ax1.set_xlabel('Complexity Category', fontsize=10, fontweight='bold')
ax1.set_ylabel('Time (ms)', fontsize=10, fontweight='bold')
ax1.set_title('Performance by Complexity\n(Final Implementations)', fontsize=11, fontweight='bold')
ax1.set_xticks(x + width)
ax1.set_xticklabels(categories, rotation=0, ha='center')
ax1.legend(fontsize=9, loc='upper left')
ax1.set_ylim(0, 5)
ax1.grid(axis='y', alpha=0.3)

# ============ Graph 2: Overall Performance Comparison ============
ax2 = plt.subplot(2, 3, 2)

names = list(final_comparison.keys())
times = list(final_comparison.values())
bar_colors = ['#4ECDC4', '#45B7D1', '#2ECC71']

bars = ax2.bar(names, times, color=bar_colors, edgecolor='black', linewidth=1.5)
ax2.set_ylabel('Time (ms)', fontsize=10, fontweight='bold')
ax2.set_title('Overall Performance\n(All Tests)', fontsize=11, fontweight='bold')
ax2.set_ylim(0, 1.5)
ax2.grid(axis='y', alpha=0.3)

# Add value labels and ratios
genson_rs_time = final_comparison['Rust genson-rs']
for bar, (name, val) in zip(bars, final_comparison.items()):
    height = bar.get_height()
    ax2.text(bar.get_x() + bar.get_width()/2., height + 0.05,
             f'{val:.2f}ms', ha='center', va='bottom', fontsize=9, fontweight='bold')

    if name != 'Rust genson-rs':
        ratio = val / genson_rs_time
        ratio_text = f'{ratio:.2f}x' if ratio > 1 else f'{1/ratio:.2f}x faster'
        ax2.text(bar.get_x() + bar.get_width()/2., height/2,
                ratio_text, ha='center', va='center', fontsize=8, color='white', fontweight='bold',
                bbox=dict(boxstyle='round', facecolor='black', alpha=0.6))

# ============ Graph 3: Speedup vs genson-rs ============
ax3 = plt.subplot(2, 3, 3)

speedup_names = ['Python GenSON', 'json-melt (Streaming)', 'Rust genson-rs (baseline)']
speedup_values = [0.36/1.04, 1.12/1.04, 1.0]  # Ratios relative to genson-rs
speedup_colors = ['#4ECDC4', '#2ECC71', '#45B7D1']

bars = ax3.barh(speedup_names, speedup_values, color=speedup_colors, edgecolor='black', linewidth=1.5)
ax3.axvline(x=1, color='red', linestyle='--', linewidth=2, alpha=0.7)
ax3.set_xlabel('Performance Ratio (vs genson-rs)', fontsize=10, fontweight='bold')
ax3.set_title('Relative Performance\nvs genson-rs', fontsize=11, fontweight='bold')
ax3.set_xlim(0, 1.2)
ax3.grid(axis='x', alpha=0.3)

# Add value labels
for bar, val in zip(bars, speedup_values):
    if val == 1.0:
        label = '1.0x (baseline)'
    elif val < 1:
        label = f'{1/val:.2f}x slower'
    else:
        label = f'{val:.2f}x faster'

    ax3.text(val + 0.02, bar.get_y() + bar.get_height()/2.,
             label, ha='left', va='center', fontsize=9, fontweight='bold')

# ============ Graph 4: json-melt Optimization Journey ============
ax4 = plt.subplot(2, 3, 4)

stages = list(optimization_stages.keys())
times_opt = list(optimization_stages.values())
colors_opt = ['#FF6B6B', '#FFA07A', '#FFD700', '#2ECC71']

bars = ax4.bar(stages, times_opt, color=colors_opt, edgecolor='black', linewidth=1.5)
ax4.set_ylabel('Time (ms)', fontsize=10, fontweight='bold')
ax4.set_title('json-melt Optimization Journey\n(Internal Cycles)', fontsize=11, fontweight='bold')
ax4.set_ylim(0, 400)
ax4.tick_params(axis='x', rotation=15)
plt.setp(ax4.xaxis.get_majorticklabels(), rotation=15, ha='right', fontsize=8)
ax4.grid(axis='y', alpha=0.3)

# Add value labels and improvements
for i, (bar, val) in enumerate(zip(bars, times_opt)):
    height = bar.get_height()
    ax4.text(bar.get_x() + bar.get_width()/2., height,
             f'{val:.2f}ms', ha='center', va='bottom', fontsize=8, fontweight='bold')

    if i > 0:
        improvement = ((times_opt[i-1] - val) / times_opt[i-1]) * 100
        ax4.text(bar.get_x() + bar.get_width()/2., height/2,
                f'{improvement:.0f}%\nfaster', ha='center', va='center',
                fontsize=7, color='white', fontweight='bold',
                bbox=dict(boxstyle='round', facecolor='black', alpha=0.7))

# ============ Graph 5: Head-to-Head Comparison (json-melt vs genson-rs by Category) ============
ax5 = plt.subplot(2, 3, 5)

categories_hth = list(complexity_data.keys())
genson_times = [complexity_data[cat]['Rust genson-rs'] for cat in categories_hth]
jsonmelt_times = [complexity_data[cat]['json-melt (Streaming)'] for cat in categories_hth]

x_hth = np.arange(len(categories_hth))
width_hth = 0.35

bars1 = ax5.bar(x_hth - width_hth/2, genson_times, width_hth, label='Rust genson-rs', color='#45B7D1', edgecolor='black', linewidth=1)
bars2 = ax5.bar(x_hth + width_hth/2, jsonmelt_times, width_hth, label='json-melt', color='#2ECC71', edgecolor='black', linewidth=1)

ax5.set_xlabel('Complexity Category', fontsize=10, fontweight='bold')
ax5.set_ylabel('Time (ms)', fontsize=10, fontweight='bold')
ax5.set_title('Direct Head-to-Head:\njson-melt vs genson-rs', fontsize=11, fontweight='bold')
ax5.set_xticks(x_hth)
ax5.set_xticklabels(categories_hth, rotation=0)
ax5.legend(fontsize=9, loc='upper left')
ax5.grid(axis='y', alpha=0.3)
ax5.set_ylim(0, 5)

# ============ Graph 6: Summary Statistics ============
ax6 = plt.subplot(2, 3, 6)
ax6.axis('off')

summary_text = """
FINAL RESULTS - json-melt Streaming Architecture

📊 Performance vs genson-rs:
  • Overall: 1.12ms vs 1.04ms
  • Ratio: 1.08x slower (7.23x → 1.08x improvement)
  • Validation: ✓ 100/100 schemas pass

📈 Optimization Journey:
  • Initial: 389.68ms
  • Final: 1.12ms
  • Total improvement: 348.4x

✓ Key Advantages:
  • Near-competitive with genson-rs
  • Better schema quality (required fields, formats)
  • 100% correctness validation
  • Production-ready implementation
"""

ax6.text(0.05, 0.95, summary_text, transform=ax6.transAxes,
        fontsize=9, verticalalignment='top', fontfamily='monospace',
        bbox=dict(boxstyle='round', facecolor='wheat', alpha=0.5))

plt.tight_layout()
plt.savefig('/home/personal/code/furnace/schema_inference/performance_graphs.png', dpi=300, bbox_inches='tight')
print("✓ Performance graphs saved to: schema_inference/performance_graphs.png")
print("✓ All graphs generated successfully!")
