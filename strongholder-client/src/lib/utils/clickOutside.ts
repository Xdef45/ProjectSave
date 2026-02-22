export function clickOutside(node: HTMLElement, callback: () => void) {
    const handleClick = (event: MouseEvent) => {
        // Si le clic est strictement en dehors du nœud et n'a pas été annulé par ailleurs
        if (node && !node.contains(event.target as Node) && !event.defaultPrevented) {
            callback();
        }
    };

    // Utilisation de la phase de capture (true) pour intercepter l'événement
    // avant qu'il ne soit potentiellement stoppé par d'autres gestionnaires.
    document.addEventListener('click', handleClick, true);

    return {
        destroy() {
            // Nettoyage impératif de l'écouteur lors de la destruction du composant
            document.removeEventListener('click', handleClick, true);
        }
    };
}