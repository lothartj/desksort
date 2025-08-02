import { invoke } from '@tauri-apps/api/tauri';

const FILE_CATEGORIES = {
    documents: ['.pdf', '.docx', '.doc', '.txt', '.odt', '.rtf'],
    spreadsheets: ['.xls', '.xlsx', '.csv', '.ods'],
    presentations: ['.pptx', '.odp', '.key'],
    images: ['.jpg', '.jpeg', '.png', '.gif', '.bmp', '.webp', '.tiff'],
    videos: ['.mp4', '.mkv', '.avi', '.mov', '.webm', '.flv', '.wmv'],
    audio: ['.mp3', '.wav', '.aac', '.ogg', '.flac'],
    archives: ['.zip', '.rar', '.7z', '.tar', '.gz', '.tar.gz'],
    executables: ['.exe', '.msi', '.sh', '.bat', '.AppImage'],
    code: ['.js', '.py', '.rs', '.cpp', '.java', '.html', '.css', '.json', '.ts'],
    folders: ['folder']
};

const sortNowBtn = document.getElementById('sortNowBtn');
const saveSettingsBtn = document.getElementById('saveSettingsBtn');
const categoryMappings = document.getElementById('categoryMappings');
const sortingStatus = document.getElementById('sortingStatus');
const logContainer = document.getElementById('logContainer');

async function initializeApp() {
    try {
        const settings = await invoke('load_settings');
        renderCategoryMappings(settings);
        addEventListeners();
        log('App initialized successfully');
    } catch (error) {
        showError('Failed to initialize app: ' + error);
    }
}

async function loadSettings() {
    try {
        return await invoke('load_settings');
    } catch (error) {
        throw new Error('Failed to load settings: ' + error);
    }
}

async function saveSettings(settings) {
    try {
        await invoke('save_settings', { settings });
        showSuccess('Settings saved successfully');
    } catch (error) {
        showError('Failed to save settings: ' + error);
    }
}

function renderCategoryMappings(settings) {
    categoryMappings.innerHTML = '';
    
    Object.entries(FILE_CATEGORIES).forEach(([category, extensions]) => {
        const div = document.createElement('div');
        div.className = 'category-mapping';
        
        const label = document.createElement('label');
        label.textContent = `${category} (${extensions.join(', ')})`;
        
        const input = document.createElement('input');
        input.type = 'text';
        input.value = settings[category] || '';
        input.placeholder = `Path for ${category}`;
        input.dataset.category = category;
        
        div.appendChild(label);
        div.appendChild(input);
        categoryMappings.appendChild(div);
    });
}

function collectSettings() {
    const settings = {};
    const inputs = categoryMappings.querySelectorAll('input');
    
    inputs.forEach(input => {
        if (input.value.trim()) {
            settings[input.dataset.category] = input.value.trim();
        }
    });
    
    return settings;
}

async function startSorting() {
    try {
        sortNowBtn.disabled = true;
        showStatus('Sorting in progress...', 'info');
        
        const result = await invoke('scan_and_sort');
        showSuccess('Sorting completed successfully');
        log('Sorting completed: ' + JSON.stringify(result));
        if (result.moved_files.length > 0) {
            result.moved_files.forEach(msg => log(msg));
        }
        if (result.errors.length > 0) {
            result.errors.forEach(err => log('Error: ' + err));
        }
    } catch (error) {
        showError('Sorting failed: ' + error);
    } finally {
        sortNowBtn.disabled = false;
    }
}
function addEventListeners() {
    sortNowBtn.addEventListener('click', startSorting);
    saveSettingsBtn.addEventListener('click', async () => {
        const settings = collectSettings();
        await saveSettings(settings);
    });
}
function showStatus(message, type) {
    sortingStatus.textContent = message;
    sortingStatus.className = `status-box ${type}`;
    sortingStatus.style.display = 'block';
}

function showSuccess(message) {
    showStatus(message, 'success');
}

function showError(message) {
    showStatus(message, 'error');
    log('Error: ' + message);
}

function log(message) {
    const entry = document.createElement('div');
    entry.className = 'log-entry';
    entry.textContent = `[${new Date().toLocaleTimeString()}] ${message}`;
    logContainer.insertBefore(entry, logContainer.firstChild);
}
document.addEventListener('DOMContentLoaded', initializeApp); 