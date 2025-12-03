/**
 * Prompts Settings Component
 *
 * Settings UI for custom AI prompts configuration.
 * Displays prompts with J-Editorial taxonomy fields.
 */

import { Setting, Notice, setIcon } from 'obsidian';
import type LabeledAnnotations from '../../main';
import { PromptLoader, BUILTIN_PROMPTS } from '../../llm';
import { PromptCategory, PromptDefinition, TaskFamily, ReliabilityTier } from '../../llm/prompt-schema';

interface Props {
    plugin: LabeledAnnotations;
    containerEl: HTMLElement;
}

export const PromptsSettings = ({ plugin, containerEl }: Props) => {
    const settings = plugin.settings.getValue();
    const promptsConfig = settings.prompts;

    // Header
    containerEl.createEl('h2', { text: 'Custom Prompts' });
    containerEl.createEl('p', {
        text: 'Configure AI prompts for document analysis. Define custom prompts in YAML files.',
        cls: 'setting-item-description',
    });

    // Prompts folder path
    new Setting(containerEl)
        .setName('Prompts Folder')
        .setDesc('Vault-relative path to folder containing custom prompt YAML files')
        .addText((text) => {
            text.inputEl.style.width = '250px';
            text.setPlaceholder('.doc-doctor/prompts')
                .setValue(promptsConfig.promptsPath)
                .onChange((value) => {
                    plugin.settings.dispatch({
                        type: 'PROMPTS_SET_PATH',
                        payload: { path: value.trim() || '.doc-doctor/prompts' },
                    });
                });
        })
        .addButton((button) => {
            button
                .setButtonText('Create')
                .setTooltip('Create the prompts folder if it does not exist')
                .onClick(async () => {
                    try {
                        const path = promptsConfig.promptsPath || '.doc-doctor/prompts';
                        const folder = plugin.app.vault.getAbstractFileByPath(path);

                        if (folder) {
                            new Notice('Folder already exists');
                        } else {
                            await plugin.app.vault.createFolder(path);
                            new Notice(`Created folder: ${path}`);
                        }
                    } catch (error) {
                        new Notice(`Error creating folder: ${error}`);
                    }
                });
        });

    // Watch for changes
    new Setting(containerEl)
        .setName('Watch for Changes')
        .setDesc('Automatically reload prompts when YAML files change')
        .addToggle((toggle) => {
            toggle.setValue(promptsConfig.watchForChanges).onChange((value) => {
                plugin.settings.dispatch({
                    type: 'PROMPTS_SET_WATCH_FOR_CHANGES',
                    payload: { enabled: value },
                });
            });
        });

    // Show built-in prompts
    new Setting(containerEl)
        .setName('Show Built-in Prompts')
        .setDesc('Include default prompts alongside your custom ones')
        .addToggle((toggle) => {
            toggle.setValue(promptsConfig.showBuiltinPrompts).onChange((value) => {
                plugin.settings.dispatch({
                    type: 'PROMPTS_SET_SHOW_BUILTIN',
                    payload: { enabled: value },
                });
            });
        });

    // Default category filter
    new Setting(containerEl)
        .setName('Default Category')
        .setDesc('Filter prompts by category in the command palette')
        .addDropdown((dropdown) => {
            dropdown
                .addOption('all', 'All Categories')
                .addOption('analysis', 'Analysis')
                .addOption('editing', 'Editing')
                .addOption('review', 'Review')
                .addOption('custom', 'Custom')
                .setValue(promptsConfig.defaultCategory)
                .onChange((value) => {
                    plugin.settings.dispatch({
                        type: 'PROMPTS_SET_DEFAULT_CATEGORY',
                        payload: { category: value as PromptCategory | 'all' },
                    });
                });
        });

    // ==========================================================================
    // BEHAVIOR DEFAULTS
    // ==========================================================================

    containerEl.createEl('h3', { text: 'Default Behaviors' });
    containerEl.createEl('p', {
        text: 'Default settings for prompts that don\'t specify their own behavior.',
        cls: 'setting-item-description',
    });

    // Confirm before apply
    new Setting(containerEl)
        .setName('Confirm Before Apply')
        .setDesc('Ask for confirmation before applying suggested changes')
        .addToggle((toggle) => {
            toggle.setValue(promptsConfig.confirmBeforeApply).onChange((value) => {
                plugin.settings.dispatch({
                    type: 'PROMPTS_SET_CONFIRM_BEFORE_APPLY',
                    payload: { enabled: value },
                });
            });
        });

    // Show preview panel
    new Setting(containerEl)
        .setName('Show Preview Panel')
        .setDesc('Display a preview of proposed changes before applying')
        .addToggle((toggle) => {
            toggle.setValue(promptsConfig.showPreviewPanel).onChange((value) => {
                plugin.settings.dispatch({
                    type: 'PROMPTS_SET_SHOW_PREVIEW_PANEL',
                    payload: { enabled: value },
                });
            });
        });

    // Auto-insert anchors
    new Setting(containerEl)
        .setName('Auto-insert Anchors')
        .setDesc('Automatically insert inline anchors (^anchor-id) at suggested locations')
        .addToggle((toggle) => {
            toggle.setValue(promptsConfig.autoInsertAnchors).onChange((value) => {
                plugin.settings.dispatch({
                    type: 'PROMPTS_SET_AUTO_INSERT_ANCHORS',
                    payload: { enabled: value },
                });
            });
        });

    // ==========================================================================
    // LOADED PROMPTS
    // ==========================================================================

    containerEl.createEl('h3', { text: 'Available Prompts' });

    // Reload button
    new Setting(containerEl)
        .setName('Reload Prompts')
        .setDesc('Reload all prompts from disk')
        .addButton((button) => {
            button
                .setButtonText('Reload')
                .onClick(async () => {
                    button.setButtonText('Loading...');
                    button.setDisabled(true);

                    try {
                        const loader = new PromptLoader(
                            plugin.app,
                            promptsConfig.promptsPath,
                            promptsConfig.showBuiltinPrompts
                        );
                        await loader.loadFromVault();
                        const prompts = loader.getAll();

                        new Notice(`Loaded ${prompts.length} prompts`);

                        // Refresh settings display
                        containerEl.empty();
                        PromptsSettings({ plugin, containerEl });
                    } catch (error) {
                        new Notice(`Error loading prompts: ${error}`);
                        console.error('[Doc Doctor] Prompt reload error:', error);
                    } finally {
                        button.setButtonText('Reload');
                        button.setDisabled(false);
                    }
                });
        });

    // Get hidden prompts from settings
    const hiddenPromptIds = promptsConfig.hiddenPromptIds || [];

    // List built-in prompts
    if (promptsConfig.showBuiltinPrompts) {
        containerEl.createEl('h4', { text: 'Built-in Prompts' });

        const builtinList = containerEl.createEl('div', {
            cls: 'doc-doctor-prompts-list',
        });

        // Filter out hidden prompts
        const visiblePrompts = BUILTIN_PROMPTS.filter(p => !hiddenPromptIds.includes(p.id));

        visiblePrompts.forEach((prompt) => {
            renderPromptCard(builtinList, prompt as PromptDefinition, plugin, () => {
                // Hide prompt
                plugin.settings.dispatch({
                    type: 'PROMPTS_HIDE_PROMPT',
                    payload: { promptId: prompt.id },
                });
                // Re-render
                containerEl.empty();
                PromptsSettings({ plugin, containerEl });
            });
        });

        // Add Prompt button (always show)
        const addPromptSection = containerEl.createDiv('dd-add-prompt-section');

        const addPromptBtn = addPromptSection.createEl('button', {
            cls: 'dd-add-prompt-btn',
        });
        setIcon(addPromptBtn.createSpan(), 'plus');
        addPromptBtn.createSpan({ text: ' Add Prompt' });

        addPromptBtn.addEventListener('click', () => {
            showAddPromptModal(plugin, hiddenPromptIds, containerEl);
        });

        // Show hidden count indicator if any
        if (hiddenPromptIds.length > 0) {
            addPromptSection.createEl('span', {
                text: `(${hiddenPromptIds.length} hidden)`,
                cls: 'dd-hidden-indicator',
            });
        }
    }

    // Show vault prompts (async load)
    const vaultPromptsSection = containerEl.createEl('div');
    vaultPromptsSection.createEl('h4', { text: 'Custom Prompts (from vault)' });

    const loadVaultPrompts = async () => {
        try {
            const loader = new PromptLoader(
                plugin.app,
                promptsConfig.promptsPath,
                false // Don't include builtins here
            );
            await loader.loadFromVault();
            const vaultPrompts = loader.getVaultPrompts();

            if (vaultPrompts.length === 0) {
                vaultPromptsSection.createEl('p', {
                    text: 'No custom prompts found. Create YAML files in the prompts folder.',
                    cls: 'setting-item-description',
                });

                // Show example
                const exampleEl = vaultPromptsSection.createEl('details');
                exampleEl.createEl('summary', { text: 'Example prompt file' });

                const codeEl = exampleEl.createEl('pre', {
                    cls: 'doc-doctor-code-block',
                });
                codeEl.createEl('code', {
                    text: `# my-prompt.yaml
id: my-custom-prompt
name: "My Custom Analysis"
category: analysis
context:
  requires_selection: false
  requires_file: true
system_extension: |
  Analyze this document for specific issues.
  Focus on:
  - Item 1
  - Item 2
behavior:
  confirm_before_apply: true
  show_preview: true`,
                });
            } else {
                const customList = vaultPromptsSection.createEl('div', {
                    cls: 'doc-doctor-prompts-list',
                });

                vaultPrompts.forEach((prompt) => {
                    const item = customList.createEl('div', {
                        cls: 'doc-doctor-prompt-item',
                    });

                    item.createEl('span', {
                        cls: 'doc-doctor-prompt-icon',
                        text: getIconForCategory(prompt.category),
                    });

                    const info = item.createEl('div', {
                        cls: 'doc-doctor-prompt-info',
                    });

                    info.createEl('span', {
                        cls: 'doc-doctor-prompt-name',
                        text: prompt.name,
                    });

                    info.createEl('span', {
                        cls: 'doc-doctor-prompt-category setting-item-description',
                        text: `${prompt.category} • ${prompt.filePath || 'unknown'}`,
                    });
                });
            }
        } catch (error) {
            vaultPromptsSection.createEl('p', {
                text: `Error loading prompts: ${error}`,
                cls: 'setting-item-description mod-warning',
            });
        }
    };

    loadVaultPrompts();

    // Add styles
    addPromptsStyles(containerEl);
};

/**
 * Render a prompt card with taxonomy info and remove button
 */
function renderPromptCard(
    containerEl: HTMLElement,
    prompt: PromptDefinition,
    plugin: LabeledAnnotations,
    onHide: () => void
): void {
    const card = containerEl.createDiv('dd-prompt-card');

    // Header row with icon, name, and remove button
    const header = card.createDiv('dd-prompt-header');

    // Icon
    const iconEl = header.createSpan('dd-prompt-icon');
    if (prompt.icon) {
        setIcon(iconEl, prompt.icon);
    } else {
        iconEl.textContent = getIconForCategory(prompt.category);
    }

    // Name and description
    const titleSection = header.createDiv('dd-prompt-title-section');
    titleSection.createEl('span', {
        cls: 'dd-prompt-name',
        text: prompt.name,
    });
    if (prompt.description) {
        titleSection.createEl('span', {
            cls: 'dd-prompt-description',
            text: prompt.description,
        });
    }

    // Remove button
    const removeBtn = header.createEl('button', {
        cls: 'dd-prompt-remove-btn',
        attr: { 'aria-label': 'Hide prompt' },
    });
    setIcon(removeBtn, 'x');
    removeBtn.addEventListener('click', (e) => {
        e.stopPropagation();
        onHide();
    });

    // Taxonomy badges row
    const badges = card.createDiv('dd-prompt-badges');

    // Category badge
    badges.createEl('span', {
        cls: `dd-badge dd-badge-category dd-badge-${prompt.category}`,
        text: prompt.category,
    });

    // Task family badge
    if (prompt.task_family) {
        badges.createEl('span', {
            cls: `dd-badge dd-badge-family dd-badge-${prompt.task_family}`,
            text: formatTaskFamily(prompt.task_family),
        });
    }

    // Vector type badge
    if (prompt.vector_type) {
        badges.createEl('span', {
            cls: 'dd-badge dd-badge-vector',
            text: prompt.vector_type,
        });
    }

    // Reliability badge
    if (prompt.reliability) {
        badges.createEl('span', {
            cls: `dd-badge dd-badge-reliability dd-badge-reliability-${prompt.reliability}`,
            text: `${prompt.reliability} reliability`,
        });
    }

    // Context info (collapsible)
    const details = card.createEl('details', { cls: 'dd-prompt-details' });
    details.createEl('summary', { text: 'Context & Behavior' });

    const detailsContent = details.createDiv('dd-prompt-details-content');

    // Context requirements
    const contextInfo = detailsContent.createDiv('dd-prompt-context');
    contextInfo.createEl('strong', { text: 'Context:' });
    const contextList = contextInfo.createEl('ul');
    contextList.createEl('li', {
        text: `Selection: ${prompt.context.requires_selection ? 'Required' : 'Optional'}`,
    });
    contextList.createEl('li', {
        text: `File: ${prompt.context.requires_file ? 'Required' : 'Optional'}`,
    });
    if (prompt.context.file_types && prompt.context.file_types.length > 0) {
        contextList.createEl('li', {
            text: `File types: ${prompt.context.file_types.join(', ')}`,
        });
    }

    // Behavior
    const behaviorInfo = detailsContent.createDiv('dd-prompt-behavior');
    behaviorInfo.createEl('strong', { text: 'Behavior:' });
    const behaviorList = behaviorInfo.createEl('ul');
    behaviorList.createEl('li', {
        text: `Confirm: ${prompt.behavior.confirm_before_apply ? 'Yes' : 'No'}`,
    });
    behaviorList.createEl('li', {
        text: `Preview: ${prompt.behavior.show_preview ? 'Yes' : 'No'}`,
    });
    behaviorList.createEl('li', {
        text: `Auto-anchors: ${prompt.behavior.auto_insert_anchors ? 'Yes' : 'No'}`,
    });

    // Hotkey if defined
    if (prompt.hotkey) {
        detailsContent.createEl('div', {
            cls: 'dd-prompt-hotkey',
            text: `Hotkey: ${prompt.hotkey}`,
        });
    }
}

/**
 * Format task family for display
 */
function formatTaskFamily(family: TaskFamily): string {
    const familyLabels: Record<TaskFamily, string> = {
        generative: 'Generative (I)',
        combinatorial: 'Combinatorial (II)',
        synoptic: 'Synoptic (III)',
        operational: 'Operational (IV)',
        learning: 'Learning (V)',
    };
    return familyLabels[family] || family;
}

/**
 * Show modal for adding prompts (restore hidden or load custom)
 */
function showAddPromptModal(
    plugin: LabeledAnnotations,
    hiddenPromptIds: string[],
    containerEl: HTMLElement
): void {
    // Find the hidden prompts from builtin list
    const hiddenPrompts = BUILTIN_PROMPTS.filter((p) => hiddenPromptIds.includes(p.id));

    // Create modal overlay
    const overlay = document.createElement('div');
    overlay.className = 'dd-modal-overlay';

    const modal = overlay.createDiv('dd-add-prompt-modal');

    // Header
    const modalHeader = modal.createDiv('dd-modal-header');
    modalHeader.createEl('h3', { text: 'Add Prompt' });

    const closeBtn = modalHeader.createEl('button', { cls: 'dd-modal-close' });
    setIcon(closeBtn, 'x');
    closeBtn.addEventListener('click', () => overlay.remove());

    // Content
    const content = modal.createDiv('dd-modal-content');

    // Section: Restore Hidden Prompts
    if (hiddenPrompts.length > 0) {
        const restoreSection = content.createDiv('dd-modal-section');
        restoreSection.createEl('h4', { text: 'Restore Hidden Prompts' });
        restoreSection.createEl('p', {
            text: 'Re-enable prompts you previously hid.',
            cls: 'setting-item-description',
        });

        const restoreList = restoreSection.createDiv('dd-restore-list');

        hiddenPrompts.forEach((prompt) => {
            const item = restoreList.createDiv('dd-restore-item');

            const info = item.createDiv('dd-restore-info');
            info.createEl('span', { cls: 'dd-restore-name', text: prompt.name });
            info.createEl('span', {
                cls: 'dd-restore-desc',
                text: prompt.description || prompt.category,
            });

            const restoreBtn = item.createEl('button', {
                cls: 'dd-restore-item-btn',
                text: 'Restore',
            });
            restoreBtn.addEventListener('click', () => {
                plugin.settings.dispatch({
                    type: 'PROMPTS_RESTORE_PROMPT',
                    payload: { promptId: prompt.id },
                });
                // Re-render
                overlay.remove();
                containerEl.empty();
                PromptsSettings({ plugin, containerEl });
            });
        });

        // Restore all button
        if (hiddenPrompts.length > 1) {
            const restoreAllBtn = restoreSection.createEl('button', {
                cls: 'dd-restore-all-btn',
                text: 'Restore All Hidden',
            });
            restoreAllBtn.addEventListener('click', () => {
                plugin.settings.dispatch({
                    type: 'PROMPTS_RESTORE_ALL',
                    payload: {},
                });
                overlay.remove();
                containerEl.empty();
                PromptsSettings({ plugin, containerEl });
            });
        }
    }

    // Section: Load Custom Prompt
    const customSection = content.createDiv('dd-modal-section');
    customSection.createEl('h4', { text: 'Load Custom Prompt' });
    customSection.createEl('p', {
        text: 'Create a YAML file in your prompts folder to add custom prompts.',
        cls: 'setting-item-description',
    });

    // Quick actions for custom prompts
    const customActions = customSection.createDiv('dd-custom-actions');

    // Open prompts folder button
    const openFolderBtn = customActions.createEl('button', {
        cls: 'dd-action-btn',
    });
    const folderIconEl = openFolderBtn.createSpan();
    setIcon(folderIconEl, 'folder-open');
    openFolderBtn.createSpan({ text: ' Open Prompts Folder' });

    openFolderBtn.addEventListener('click', async () => {
        const promptsPath = plugin.settings.getValue().prompts.promptsPath || '.doc-doctor/prompts';
        const folder = plugin.app.vault.getAbstractFileByPath(promptsPath);

        if (!folder) {
            // Create folder if it doesn't exist
            try {
                await plugin.app.vault.createFolder(promptsPath);
                new Notice(`Created folder: ${promptsPath}`);
            } catch (e) {
                new Notice(`Could not create folder: ${e}`);
                return;
            }
        }

        // Open in system explorer (using Obsidian's built-in method)
        // @ts-ignore - showInFolder is available but not in types
        if (plugin.app.showInFolder) {
            // @ts-ignore
            plugin.app.showInFolder(promptsPath);
        } else {
            new Notice(`Prompts folder: ${promptsPath}`);
        }
        overlay.remove();
    });

    // Create from template button
    const templateBtn = customActions.createEl('button', {
        cls: 'dd-action-btn',
    });
    const templateIconEl = templateBtn.createSpan();
    setIcon(templateIconEl, 'file-plus');
    templateBtn.createSpan({ text: ' Create from Template' });

    templateBtn.addEventListener('click', async () => {
        const promptsPath = plugin.settings.getValue().prompts.promptsPath || '.doc-doctor/prompts';

        // Ensure folder exists
        const folder = plugin.app.vault.getAbstractFileByPath(promptsPath);
        if (!folder) {
            try {
                await plugin.app.vault.createFolder(promptsPath);
            } catch (e) {
                new Notice(`Could not create folder: ${e}`);
                return;
            }
        }

        // Create template file
        const templateContent = `# Custom Prompt Template
# Copy this file and customize for your use case

id: my-custom-prompt
name: "My Custom Prompt"
description: "Brief description of what this prompt does"
category: custom  # analysis | editing | review | custom
icon: sparkles    # Lucide icon name

# Task taxonomy (optional but recommended)
task_family: combinatorial  # generative | combinatorial | synoptic | operational | learning
vector_type: check          # draft | cut | idea | source | check | link | data | model | fix | move
reliability: medium         # high | medium | low

context:
  requires_selection: false
  requires_file: true
  file_types:
    - md

system_extension: |
  Your custom instructions go here.

  The AI will use these instructions when this prompt is selected.
  You can include:
  - Specific analysis criteria
  - Output format requirements
  - Domain-specific guidance

behavior:
  confirm_before_apply: true
  auto_insert_anchors: true
  show_preview: true

# Optional hotkey (Obsidian format)
# hotkey: "Mod+Shift+P"
`;

        const fileName = `${promptsPath}/custom-prompt-template.yaml`;
        try {
            await plugin.app.vault.create(fileName, templateContent);
            new Notice(`Created template: ${fileName}`);
            // Open the file
            const file = plugin.app.vault.getAbstractFileByPath(fileName);
            if (file) {
                plugin.app.workspace.openLinkText(fileName, '', true);
            }
        } catch (e) {
            new Notice(`Could not create template: ${e}`);
        }

        overlay.remove();
    });

    // Example format
    const exampleDetails = customSection.createEl('details', { cls: 'dd-example-details' });
    exampleDetails.createEl('summary', { text: 'View YAML format example' });

    const exampleCode = exampleDetails.createEl('pre', { cls: 'doc-doctor-code-block' });
    exampleCode.createEl('code', {
        text: `id: fact-checker
name: "Fact Check"
category: review
task_family: combinatorial
vector_type: check
reliability: high
context:
  requires_selection: true
  requires_file: true
system_extension: |
  Verify all factual claims in the selection.
  For each claim, provide:
  1. The claim
  2. Verification status
  3. Source if available
behavior:
  confirm_before_apply: false
  show_preview: true`,
    });

    // Close on overlay click
    overlay.addEventListener('click', (e) => {
        if (e.target === overlay) overlay.remove();
    });

    document.body.appendChild(overlay);
}

/**
 * Show modal for restoring hidden prompts (legacy - kept for compatibility)
 */
function showRestorePromptsModal(
    plugin: LabeledAnnotations,
    hiddenPromptIds: string[],
    containerEl: HTMLElement
): void {
    showAddPromptModal(plugin, hiddenPromptIds, containerEl);
}

/**
 * Get icon for prompt category
 */
function getIconForCategory(category: PromptCategory): string {
    switch (category) {
        case 'analysis':
            return '🔍';
        case 'editing':
            return '✏️';
        case 'review':
            return '✓';
        case 'custom':
            return '⚙️';
        default:
            return '📄';
    }
}

/**
 * Add styles for prompts list
 */
function addPromptsStyles(containerEl: HTMLElement) {
    const styleId = 'doc-doctor-prompts-settings-styles';
    if (document.getElementById(styleId)) return;

    const style = document.createElement('style');
    style.id = styleId;
    style.textContent = `
        /* Prompts List */
        .doc-doctor-prompts-list {
            display: flex;
            flex-direction: column;
            gap: 8px;
            margin: 8px 0;
        }

        /* Prompt Card */
        .dd-prompt-card {
            background: var(--background-secondary);
            border-radius: 8px;
            padding: 12px;
            border: 1px solid var(--background-modifier-border);
        }

        .dd-prompt-header {
            display: flex;
            align-items: flex-start;
            gap: 10px;
        }

        .dd-prompt-icon {
            display: flex;
            align-items: center;
            justify-content: center;
            width: 28px;
            height: 28px;
            background: var(--background-primary);
            border-radius: 6px;
            flex-shrink: 0;
        }

        .dd-prompt-icon svg {
            width: 16px;
            height: 16px;
        }

        .dd-prompt-title-section {
            flex: 1;
            display: flex;
            flex-direction: column;
            gap: 2px;
            min-width: 0;
        }

        .dd-prompt-name {
            font-weight: 600;
            color: var(--text-normal);
        }

        .dd-prompt-description {
            font-size: 0.85em;
            color: var(--text-muted);
            line-height: 1.3;
        }

        .dd-prompt-remove-btn {
            background: transparent;
            border: none;
            cursor: pointer;
            padding: 4px;
            border-radius: 4px;
            color: var(--text-muted);
            opacity: 0.5;
            transition: all 0.15s ease;
        }

        .dd-prompt-remove-btn:hover {
            opacity: 1;
            color: var(--text-error);
            background: var(--background-modifier-error);
        }

        .dd-prompt-remove-btn svg {
            width: 14px;
            height: 14px;
        }

        /* Taxonomy Badges */
        .dd-prompt-badges {
            display: flex;
            flex-wrap: wrap;
            gap: 6px;
            margin-top: 10px;
        }

        .dd-badge {
            font-size: 0.75em;
            padding: 2px 8px;
            border-radius: 10px;
            font-weight: 500;
            text-transform: capitalize;
        }

        .dd-badge-category {
            background: var(--background-modifier-border);
            color: var(--text-muted);
        }

        .dd-badge-analysis { background: #e3f2fd; color: #1565c0; }
        .dd-badge-editing { background: #fff3e0; color: #ef6c00; }
        .dd-badge-review { background: #e8f5e9; color: #2e7d32; }
        .dd-badge-custom { background: #f3e5f5; color: #7b1fa2; }

        .dd-badge-family {
            background: var(--background-primary);
            border: 1px solid var(--background-modifier-border);
        }

        .dd-badge-generative { color: #7c4dff; }
        .dd-badge-combinatorial { color: #00bcd4; }
        .dd-badge-synoptic { color: #ff9800; }
        .dd-badge-operational { color: #4caf50; }
        .dd-badge-learning { color: #e91e63; }

        .dd-badge-vector {
            background: var(--background-primary);
            border: 1px solid var(--background-modifier-border);
            font-family: var(--font-monospace);
        }

        .dd-badge-reliability {
            background: var(--background-primary);
        }

        .dd-badge-reliability-high { color: #2e7d32; border: 1px solid #81c784; }
        .dd-badge-reliability-medium { color: #f57c00; border: 1px solid #ffb74d; }
        .dd-badge-reliability-low { color: #c62828; border: 1px solid #ef9a9a; }

        /* Collapsible Details */
        .dd-prompt-details {
            margin-top: 10px;
            border-top: 1px solid var(--background-modifier-border);
            padding-top: 8px;
        }

        .dd-prompt-details summary {
            font-size: 0.85em;
            color: var(--text-muted);
            cursor: pointer;
            user-select: none;
        }

        .dd-prompt-details summary:hover {
            color: var(--text-normal);
        }

        .dd-prompt-details-content {
            padding: 10px 0;
            font-size: 0.85em;
        }

        .dd-prompt-details-content ul {
            margin: 4px 0;
            padding-left: 20px;
        }

        .dd-prompt-details-content li {
            margin: 2px 0;
            color: var(--text-muted);
        }

        .dd-prompt-hotkey {
            margin-top: 8px;
            padding: 6px 10px;
            background: var(--background-primary);
            border-radius: 4px;
            font-family: var(--font-monospace);
            font-size: 0.9em;
        }

        /* Hidden Prompts Section */
        .doc-doctor-hidden-prompts {
            display: flex;
            align-items: center;
            gap: 12px;
            margin-top: 12px;
            padding: 10px 14px;
            background: var(--background-primary);
            border-radius: 6px;
            border: 1px dashed var(--background-modifier-border);
        }

        .doc-doctor-hidden-count {
            font-size: 0.85em;
            color: var(--text-muted);
        }

        .doc-doctor-restore-btn {
            background: transparent;
            border: none;
            color: var(--interactive-accent);
            cursor: pointer;
            font-size: 0.85em;
            padding: 4px 8px;
            border-radius: 4px;
        }

        .doc-doctor-restore-btn:hover {
            background: var(--interactive-accent);
            color: var(--text-on-accent);
        }

        /* Restore Modal */
        .dd-modal-overlay {
            position: fixed;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            background: rgba(0, 0, 0, 0.5);
            display: flex;
            align-items: center;
            justify-content: center;
            z-index: 1000;
        }

        .dd-restore-modal {
            background: var(--background-primary);
            border-radius: 12px;
            width: 400px;
            max-width: 90vw;
            max-height: 80vh;
            overflow: hidden;
            display: flex;
            flex-direction: column;
            box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
        }

        .dd-modal-header {
            display: flex;
            align-items: center;
            justify-content: space-between;
            padding: 16px 20px;
            border-bottom: 1px solid var(--background-modifier-border);
        }

        .dd-modal-header h3 {
            margin: 0;
            font-size: 1.1em;
        }

        .dd-modal-close {
            background: transparent;
            border: none;
            cursor: pointer;
            padding: 4px;
            border-radius: 4px;
            color: var(--text-muted);
        }

        .dd-modal-close:hover {
            background: var(--background-modifier-hover);
            color: var(--text-normal);
        }

        .dd-restore-list {
            flex: 1;
            overflow-y: auto;
            padding: 12px 20px;
        }

        .dd-restore-item {
            display: flex;
            align-items: center;
            justify-content: space-between;
            padding: 10px 12px;
            background: var(--background-secondary);
            border-radius: 6px;
            margin-bottom: 8px;
        }

        .dd-restore-info {
            display: flex;
            flex-direction: column;
            gap: 2px;
        }

        .dd-restore-name {
            font-weight: 500;
        }

        .dd-restore-desc {
            font-size: 0.85em;
            color: var(--text-muted);
        }

        .dd-restore-item-btn {
            background: var(--interactive-accent);
            color: var(--text-on-accent);
            border: none;
            padding: 4px 12px;
            border-radius: 4px;
            cursor: pointer;
            font-size: 0.85em;
        }

        .dd-restore-item-btn:hover {
            filter: brightness(1.1);
        }

        .dd-modal-footer {
            padding: 12px 20px;
            border-top: 1px solid var(--background-modifier-border);
            display: flex;
            justify-content: flex-end;
        }

        /* Code Block */
        .doc-doctor-code-block {
            background: var(--background-secondary);
            padding: 12px;
            border-radius: 6px;
            overflow-x: auto;
            font-size: 0.85em;
            margin: 8px 0;
        }

        /* Add Prompt Section */
        .dd-add-prompt-section {
            display: flex;
            align-items: center;
            gap: 12px;
            margin-top: 12px;
            padding-top: 12px;
            border-top: 1px dashed var(--background-modifier-border);
        }

        .dd-add-prompt-btn {
            display: flex;
            align-items: center;
            gap: 6px;
            padding: 8px 16px;
            background: transparent;
            border: 1px dashed var(--interactive-accent);
            color: var(--interactive-accent);
            border-radius: 6px;
            cursor: pointer;
            font-size: 0.9em;
            transition: all 0.15s ease;
        }

        .dd-add-prompt-btn:hover {
            background: var(--interactive-accent);
            color: var(--text-on-accent);
            border-style: solid;
        }

        .dd-add-prompt-btn svg {
            width: 14px;
            height: 14px;
        }

        .dd-hidden-indicator {
            font-size: 0.85em;
            color: var(--text-muted);
        }

        /* Add Prompt Modal */
        .dd-add-prompt-modal {
            background: var(--background-primary);
            border-radius: 12px;
            width: 480px;
            max-width: 90vw;
            max-height: 85vh;
            overflow: hidden;
            display: flex;
            flex-direction: column;
            box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
        }

        .dd-modal-content {
            flex: 1;
            overflow-y: auto;
            padding: 0 20px 20px;
        }

        .dd-modal-section {
            padding: 16px 0;
            border-bottom: 1px solid var(--background-modifier-border);
        }

        .dd-modal-section:last-child {
            border-bottom: none;
        }

        .dd-modal-section h4 {
            margin: 0 0 4px 0;
            font-size: 0.95em;
            color: var(--text-normal);
        }

        .dd-modal-section > p {
            margin: 0 0 12px 0;
        }

        .dd-restore-all-btn {
            margin-top: 8px;
            background: transparent;
            border: 1px solid var(--background-modifier-border);
            color: var(--text-muted);
            padding: 6px 12px;
            border-radius: 4px;
            cursor: pointer;
            font-size: 0.85em;
        }

        .dd-restore-all-btn:hover {
            background: var(--background-secondary);
            color: var(--text-normal);
        }

        .dd-custom-actions {
            display: flex;
            flex-direction: column;
            gap: 8px;
        }

        .dd-action-btn {
            display: flex;
            align-items: center;
            gap: 8px;
            padding: 10px 14px;
            background: var(--background-secondary);
            border: 1px solid var(--background-modifier-border);
            border-radius: 6px;
            cursor: pointer;
            color: var(--text-normal);
            transition: all 0.15s ease;
        }

        .dd-action-btn:hover {
            background: var(--interactive-accent);
            color: var(--text-on-accent);
            border-color: var(--interactive-accent);
        }

        .dd-action-btn svg {
            width: 16px;
            height: 16px;
        }

        .dd-example-details {
            margin-top: 12px;
        }

        .dd-example-details summary {
            font-size: 0.85em;
            color: var(--text-muted);
            cursor: pointer;
        }

        .dd-example-details summary:hover {
            color: var(--text-normal);
        }
    `;
    containerEl.ownerDocument.head.appendChild(style);
}
