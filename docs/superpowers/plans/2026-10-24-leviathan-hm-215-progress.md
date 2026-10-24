# SDD ledger — plan: docs/superpowers/plans/2026-10-24-leviathan-hm-215.md

- Baseline : `cargo test --workspace --locked` → 70 tests réussis.
- Méthode : implémentation dans la session demandée explicitement ; aucun commit/push automatique, journal versionnable à la place des scripts shell Unix du skill.
- Pré-flight 1→2→3 : maxima absolus et scaling avant capacités et HUD ; pas de conflit.
- Pré-flight 2→4 : rendu des dangers basé sur leur échéance de simulation ; pas de conflit.
- Pré-flight 3→5 : les validations doivent employer l'état du passager et non uniquement le véhicule ; adapter les aides de rendu.
- Recherche : script AzerothCore moderne confirme Ulduar scaling total15 slots épiques, pas l'ancien coefficient générique 1%/ilvl. À 215 : 1.45.

## Exécution

- Branche locale `feat/leviathan-hm-215`, aucun commit, merge ou push automatique ; la documentation de conception non suivie préexistante est conservée.
- Task 1 : table de référence dans `docs/wotlk-reference.md`, révision AzerothCore figée, PV/scaling et maxima absolus ; régression PV observée RED puis GREEN.
- Task 2 : coûts, GCD et ressources par poste, accélération payante du passager IA, vitesse7/retour caméraNon, pyrite, dégâts/tours/boss et arène agrandie. Les tests boost et GCD ont échoué avant correction ; les anciennes assertions de valeurs normalisées ont été adaptées sans retirer leurs contrôles.
- Task 3 : bouton contextuel de tourelles, santé du passager séparée, surcharge, éjection et secours ; tests simulation et client RED→GREEN pour le bouton et les maxima HUD.
- Task 4 : importer des plantes Idle/Drive/Attack (test Python RED→GREEN), GLB copiés depuis les MPQ réels, secours végétal, rayons verticaux/impacts/glace. Premier essai GPU passager100s : PASS, 24 scènes végétales chargées et une surcharge, mais ce binaire précédait les corrections de revue ci-dessous et ne vaut pas validation finale.
- Ruling: les variantes non certifiées ne sont pas inventées comme « exactes » — napalm, ruée, moyennes et physique restent explicitement des profils dans la référence et le README — coût : la demande de reproduction intégrale exacte reste inachevée, même si les fonctionnalités principales sont livrées.

## Revue fraîche et correction unique

Revue en lecture seule par un autre agent, retour : six défauts importants, aucune alerte critique. Vérifiés dans le code puis par régressions échouant avant correction.

- Final: fixed horloge des tourelles accumulée sans cible — `turrets_do_not_bank_attacks_before_a_late_boarding` RED→GREEN.
- Final: fixed déplacement au sol bloqué par la collecte des commandes — `ground_passenger_can_move_but_boarded_and_carried_passengers_cannot` RED→GREEN ; intégration de mouvement/rotation dans `raid::after_step` vérifiée.
- Final: fixed invulnérabilité des passagers exposés aux zones et Décharges — `ground_and_carried_passengers_are_not_immune_to_fire` RED→GREEN ; ciblage des plantes sur les joueurs au sol confirmé par `freyas_plants_attack_a_nearer_player_on_the_ground` RED→GREEN.
- Final: fixed capacités hors GCD bloquées et effaçant le GCD courant — `off_gcd_mortar_ignores_but_preserves_the_active_passenger_gcd` RED→GREEN, cible du test corrigée pour être réellement à portée50.
- Final: fixed défaite liée au véhicule d'origine pendant le vol — `passenger_survives_original_vehicle_death_during_flight` RED→GREEN.
- Final: fixed transport bloqué après destruction d'un véhicule — `carrier_death_releases_the_passenger_to_the_ground` et `motorcycle_can_drop_a_passenger_whose_home_was_destroyed` RED→GREEN ; pas de réembarquement des morts.
- Final: fixed ancienne limite ±50 du placement IA — classée importante pour le fonctionnement dans la nouvelle arène ; `offensive_ai_can_position_in_the_expanded_half_of_the_arena` RED→GREEN. Régression des quatre coins déplacée aux coins réels ±103.
- Final: fixed README/aide/guide de livraison et étiquettes des PV absolus affichées en pourcentages — nécessaire à la livraison demandée, pas une amélioration optionnelle.
- Final: deferred minors : aucun. Les écarts de fidélité sont des manques de périmètre importants, pas des détails cosmétiques à masquer.
- Final: Ruling: fidélité privée ChromieCraft et shader original non jugés par le relecteur — aucune affirmation de certification ; limites explicites dans le guide — coût si la référence diffère : difficulté/visuels différents du serveur.
- Final: contrôles GPU/autonomie/performance non jugés par le relecteur — pris en charge par les essais natifs, pas par sa revue statique.
- Suite finale après corrections : `cargo test --workspace --locked` → **99 tests Rust réussis**, build MSVC optimisé et copie de livraison via `Construire.ps1 -SansTests` réussis, `dist/LeviathanRust-0.3.exe` ; réglages non liés conservés.

## Vérification finale

- [x] Tests Rust et compilation après corrections de revue : 99/99.
- [x] Formatage et tests Python avec MPQ réel réexécutés : 14/14.
- [x] Cinq postes GPU (20s, passager100s) après les six premiers correctifs. Passager101s rejoué après le dernier correctif plantes ; quatre essais20s non rejoués (pas de plantes à20s).
- [x] Menu, secours procédural, autonomie hors WoW sur le dernier binaire ; lancement depuis C:\Windows, copie960×640, empreinte des réglages source conservée.
- [x] Rapports finaux et limites consignés dans le README, le guide de livraison et le bilan utilisateur.

Résultats natifs finaux : `validation-hm215-demo-passenger.*` → PASS, temps101,1s,54commandes,1surcharge,24scènes végétales ; FPS lissés61 sur RTX4070 (mesure ponctuelle, pas un benchmark). `validation-menu.*`, `validation-fallback.*`, `validation-hm215-standalone.*` → PASS. Aucun plantage ni erreur de chargement observé dans ces essais.

Limites non levées : fidélité intégrale des valeurs/variantes serveur, charge physique du siège, rotations/classes et armure/hasard complets, inspection humaine exhaustive des effets et secours de plantes sur panne GPU asynchrone. Le profil actuel ne doit pas être présenté comme une reproduction complète certifiée.

Empreinte du binaire livré identique au build vérifié : SHA256 `854A6780D5CE78C1B85C096741CFB8255B7C9C6C500A00B096C1ABC6857B9A74`. Les 40 sorties du manifeste ont été revérifiées dans `assets/wow` et `dist/assets/wow` ; défauts livrés7/false confirmés. Aucun commit/push, ni modification du client WoW.
