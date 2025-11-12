

/**
 * Converts a nested config object to an INI-style string (Git config format).
 * @param configObj - The configuration object with sections, subsections, and settings.
 * @returns A formatted string representing the config.
 * @throws Error if configObj is invalid (e.g., missing sections).
 */
export function objToStr(configObj: any): string {

    // Step 1: Validate input 
    // Step 2: Extract sections and flatten into entry objects
    //         Type: Array<{ section: string; subsection: string }>
    // Step 3: Map each entry to a formatted section string
    // Step 4: Join all sections into final string

    if (!configObj || typeof configObj !== 'object' || Object.keys(configObj).length === 0) {
        throw new Error('Invalid configObj: Must be a non-empty object.');
    }
    // TODO: Learn more about it
    const entries = Object.keys(configObj).reduce((arr: { section: string; subsection: string }[], section: string) => {
        const subsections = configObj[section];
        if (!subsections || typeof subsections !== 'object') {
            throw new Error(`Invalid section '${section}': Must contain subsection objects.`);
        }
        return arr.concat(
            Object.keys(subsections).map((subsection: string) => ({ section, subsection }))
        );
    }, []);

    const formattedSections = entries.map((entry) => {
        const { section, subsection } = entry;

        // Handle subsection quoting: "" becomes empty, else " \"subname\""
        const quotedSubsection = subsection === '' ? '' : ` "${subsection}"`;

        // Retrieve settings (with optional chaining for safety)
        const settings = configObj[section][subsection];
        if (!settings || typeof settings !== 'object') {
            throw new Error(`Invalid settings for [${section}${quotedSubsection}]: Must be an object.`);
        }

        // Build header: e.g., "[core]\n" or "[core \"repository\"]\n"
        const header = `[${section}${quotedSubsection}]\n`;

        // Build settings lines: e.g., "  key = value\n"
        const settingsLines = Object.keys(settings)
            .map((key: string) => {
                const value = settings[key];
                // Coerce to string; for primitives, this works. For complex types, you'd serialize properly.
                const stringValue = typeof value === 'object' ? JSON.stringify(value) : String(value);
                return `  ${key} = ${stringValue}`;
            })
            .join('\n');


        return `${header}${settingsLines}\n`;
    });


    return formattedSections.join('');
}