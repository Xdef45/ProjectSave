// index.js
// 🚫 Ce fichier ne doit pas recréer de fenêtre Electron
// Il sert uniquement au code côté interface (DOM, navigation, etc.)

console.log("Interface STRONG HOLDER chargée ✅");

// Exemple : si tu veux que le bouton “Connexion” mène vers page2.html
document.addEventListener('DOMContentLoaded', () => {
    const btn = document.querySelector('.btn--primary');
    if (btn) {
        btn.addEventListener('click', (e) => {
            e.preventDefault();
            window.location.href = 'page2.html';
        });
    }
});
