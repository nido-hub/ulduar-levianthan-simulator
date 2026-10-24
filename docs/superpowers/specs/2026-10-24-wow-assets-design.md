# Ressources WoW locales, livraison autonome

L'utilisateur autorise la copie des ressources de son client WoW 3.3.5a et demande l'implémentation directe de l'approche proposée : convertir les véhicules en GLB et les images en PNG, conserver Rust/Bevy et la simulation actuelle.

## Objectifs et contraintes
- Importer les silhouettes et textures du Léviathan, du démolisseur, du siège et de la moto depuis les MPQ locaux, ouverts en lecture seule.
- Habiller le HUD avec les ressources locales de WoW : icônes, cadres, barres et police si disponible. Garder commandes, descriptions, coûts et recharges accessibles.
- Livrer les fichiers convertis dans `dist/assets`; aucun accès au client, outil de conversion ou réseau pendant une partie.
- Préserver les réglages et les modèles originaux de secours. Ne pas copier les archives complètes.
- Conversion ciblée 3.3.5a, pas un lecteur universel ni une reproduction des shaders propriétaires. Documenter les limites d'animation et de matériaux réellement rencontrées.

## Approches
1. **Retenue par l'utilisateur :** conversion préalable M2/SKIN/BLP vers GLB/PNG. Compatible avec le moteur et le script de livraison existants.
2. Lecture des formats WoW en jeu : rejetée, davantage de code et de dépendances sans bénéfice pour l'autonomie.

## Vérification
Tests des parseurs et de l'autonomie des GLB, tests Rust existants, captures GPU des différents véhicules et du menu. L'origine Blizzard reste documentée : la copie technique n'accorde aucun droit de redistribution.
