/**
 * Converts a nested config object to an INI-like string suitable for Git config.
 * @param configObj - The configuration object with sections, subsections, and key-value pairs.
 * @returns String representation in INI format.
 * @throws Error if the object structure is invalid.
 */
export function objToStr(configObj: any): string {
    // Validate input
    if (!configObj || typeof configObj !== 'object' || Object.keys(configObj).length === 0) {
        throw new Error('Invalid configObj: Must be a non-empty object.');
    }

    // Convert each section/subsection to an array of objects for processing
    const entries = Object.keys(configObj).reduce((arr: { section: string; subsection: string }[], section: string) => {
        const subsections = configObj[section];
        if (!subsections || typeof subsections !== 'object') {
            throw new Error(`Invalid section '${section}': Must contain subsection objects.`);
        }
        // Map each subsection to a uniform object with section and subsection keys
        return arr.concat(
            Object.keys(subsections).map((subsection: string) => ({ section, subsection }))
        );
    }, []);

    // Map each section/subsection to its INI representation
    const formattedSections = entries.map((entry) => {
        const { section, subsection } = entry;

        // Properly quote subsection name if not empty
        const quotedSubsection = subsection === '' ? '' : ` "${subsection}"`;

        // Extract settings object for this section/subsection
        const settings = configObj[section][subsection];
        if (!settings || typeof settings !== 'object') {
            throw new Error(`Invalid settings for [${section}${quotedSubsection}]: Must be an object.`);
        }

        // Compose section header: e.g., "[core]" or "[core "repository"]"
        const header = `[${section}${quotedSubsection}]\n`;

        // Generate key=value lines, serializing objects as JSON strings for readability
        const settingsLines = Object.keys(settings)
            .map((key: string) => {
                const value = settings[key];
                const stringValue = typeof value === 'object' ? JSON.stringify(value) : String(value);
                return `  ${key} = ${stringValue}`;
            })
            .join('\n');

        return `${header}${settingsLines}\n`;
    });

    // Join all sections with proper spacing
    return formattedSections.join('');
}
