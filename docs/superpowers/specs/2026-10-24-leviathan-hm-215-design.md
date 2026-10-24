# Léviathan des flammes — reproduction du combat 10 joueurs, HM, ilvl 215

Statut : conception approuvée par la demande « passe à l'implémentation » ; exécution dans la session courante.

## Suivi de conception

- [x] Lire le combat, ses entrées, son rendu, le HUD et les réglages existants.
- [x] Présenter les approches et recueillir le choix : reproduction complète.
- [x] Confirmer la référence : HM 10 joueurs, quatre tours, ilvl 215.
- [x] Rédiger la conception et signaler les limites de fidélité.
- [x] Faire relire la conception par un autre agent et traiter ses questions : aucune question bloquante relevée.
- [x] Obtenir l'approbation de cette conception.
- [x] Écrire le plan d'implémentation après approbation.

État de livraison : version jouable mise à jour et vérifiée ; la reproduction intégrale exacte reste partielle. Voir `../plans/2026-10-24-leviathan-hm-215-progress.md` et `../../wotlk-reference.md` pour les contrôles et écarts non levés.

## Intention et périmètre

Reproduire les règles du combat du Léviathan des flammes WotLK 3.3.5a à dix joueurs dans l'application Rust/Bevy existante : pas allonger artificiellement un combat normalisé. Tous les occupants simulés ont un équipement de référence de niveau moyen 215. Les quatre tours sont actives par défaut. Le mode sans tours reste accessible comme aujourd'hui.

Demandes explicites : vitesse par défaut de 7 unités/s, accélération du passager IA lorsque le joueur conduit le démolisseur, phase jouable après projection avec un seul bouton de DPS des tourelles, équilibrage complet HM, meilleurs rayons de glace/tonnerre et modèles des plantes, retour caméra réglé sur Non par défaut mais réactivable.

« Complète » porte ici sur **le combat dans l'arène**, ses véhicules et ses mécaniques. Cela n'ajoute pas le parcours avant le boss, les autres boss d'Ulduar, le multijoueur, les récompenses, les hauts faits ni une simulation générale des classes WoW. Ces exclusions sont des propositions de périmètre à approuver, pas des choix déjà exprimés par l'utilisateur. Une classe complète à pied contredirait la demande d'un bouton unique : le DPS à pied sera un profil explicite de tireur à distance ilvl 215, et non une rotation de classe.

## Approches considérées

1. **Corrections ciblées**, proposées initialement : réparer les deux blocages, ajuster les PV, améliorer les effets sans remplacer les valeurs pédagogiques. Plus rapide, mais conserve des erreurs de règles et de difficulté. Non retenu par l'utilisateur.
2. **Reproduction complète du combat**, retenue par l'utilisateur : remplacer les valeurs et règles normalisées par des données identifiées et testables, avec un profil d'équipement fixe à 215 ; conserver le moteur, les cinq postes et le lancement autonome. Plus large : les entrées, le HUD, l'IA, l'arène, les validations et la livraison doivent suivre les changements du simulateur.

Pas de nouveau moteur, framework, protocole réseau ni lecteur universel de données WoW en jeu. Réutiliser les modules et le convertisseur d'assets existants.

## Référence et niveau de preuve

Référence proposée : règles WotLK 3.3.5a du combat à dix joueurs, pas mélange silencieux avec des valeurs rééquilibrées de WotLK Classic ou des réglages privés ChromieCraft non connus. Les données du client local peuvent confirmer les sorts et les ressources graphiques ; elles ne prouvent pas à elles seules les PV ni les comportements du serveur.

Sources déjà consultées le 24 octobre 2026 :

- Script d'AzerothCore, `src/server/scripts/Northrend/Ulduar/Ulduar/boss_flame_leviathan.cpp` : https://github.com/azerothcore/azerothcore-wotlk/blob/master/src/server/scripts/Northrend/Ulduar/Ulduar/boss_flame_leviathan.cpp
- Guide de comparaison, non autorité finale pour 3.3.5 : https://www.warcrafttavern.com/wotlk/guides/flame-leviathan-strategy-guide-ulduar-10/
- Archives MPQ locales, ouvertes uniquement en lecture via l'outillage existant, pour les données client 3.3.5a et les modèles.

Les sources ne sont pas parfaitement interchangeables. Le guide comporte par exemple deux descriptions contradictoires du dégel ; le script distingue des temporisations qu'un résumé de guide arrondit. Une branche actuelle d'AzerothCore est une référence d'émulation, pas la preuve de chaque valeur historique Blizzard ni de la version déployée par ChromieCraft.

**Première étape obligatoire du futur plan :** constituer une table de référence sourcée et figée, avec version/empreinte des données utilisées, identifiants des sorts et unités. Pour chaque mécanique : valeur brute, formule, fréquence, portée, école, coût, recharge/GCD, source, divergence et décision motivée. Confirmer notamment les PV de base/HM et le cumul des bonus de tours, les valeurs des tourelles, le scaling du conducteur et du passager à 215, les bonus physiques/feu et résistances, les durées de shutdown et de redémarrage. Ne pas annoncer une valeur exacte avant cette étape.

La table doit permettre de recalculer le profil 215 indépendamment. Si une donnée critique reste introuvable ou si deux variantes changent sensiblement la difficulté, suspendre cette partie et présenter la divergence au propriétaire. Aucun coefficient choisi uniquement pour atteindre une durée de kill souhaitée ne doit se faire passer pour une valeur WotLK.

## Modèle de simulation

### Valeurs et équipement

- PV courants et maxima explicites pour boss, véhicules, passagers et tourelles ; plus de maxima implicites 100/1000 disséminés dans l'interface et les tests.
- Dégâts en unités cohérentes avec ces PV. Identifier les écoles afin d'appliquer les bonus des tours, les résistances, boucliers et conditions de dégel aux bonnes attaques, une seule fois.
- Profil fixe ilvl 215 partagé par les dix occupants : pas de nouvel écran d'inventaire ni de curseur d'ilvl dans cette demande. La formule source distinguera les PV issus du conducteur et les dégâts des capacités du conducteur/passager. Si l'ilvl seul ne détermine pas les données sources (qualité/slots), expliciter le profil retenu dans la table.
- Le profil de DPS sur tourelles est documenté séparément : l'ilvl ne définit pas à lui seul un DPS de classe. Sa cadence/dégâts et sa survie doivent être justifiés ; aucun bonus caché destiné à garantir une victoire.

### Véhicules et capacités

- Conserver les deux démolisseurs, deux sièges, deux motos et dix occupants, ainsi que les cinq postes sélectionnables.
- Vérifier toutes les capacités du combat : dégâts, zones directes/périphériques, portée, coût, ressource, recharge, temps de lancement et GCD applicables. Conserver les validations avant consommation et les ressources indépendantes des deux postes.
- Reproduire la pyrite : réserve, coût des barils, dégâts directs et périodiques, empilement/rafraîchissement/expiration par lanceur, caisses aériennes puis au sol, récupération et absence du passager. Ne pas renouveler automatiquement les munitions sans collecte.
- Reproduire le goudron et son embrasement, les boucliers, les accélérations propres à chaque véhicule, les soins et transports des motos, les interruptions et leur verrouillage si applicable. Les motos doivent aussi accomplir leur rôle de goudron lorsqu'elles sont IA.
- Distinguer conducteur et passager IA : le passager IA du démolisseur utilise son propre boost lorsque le véhicule est poursuivi et qu'il est présent, disponible et ravitaillé, même si le joueur conduit. Il paie le même coût et subit les mêmes durées/recharges que le joueur. Il n'agit pas depuis la catapulte, le boss ou le sol.
- Ne jamais déclencher automatiquement les sorts du poste humain. Aucun contournement gratuit des coûts, portées ou visées pour les alliés IA.

### Conduite, distances et arène

- Vitesse de déplacement de référence par défaut : **7 u/s**. Partager les règles de vitesse de base et de bonus entre IA et joueur ; remplacer les valeurs de vitesse pédagogiques indépendantes. Les différences de véhicule ne subsistent que si elles sont sourcées et documentées.
- Une unité monde représente une unité de portée de la simulation. Vérifier les dimensions de l'arène, les rayons de collision de gameplay, les portées et les trajets des tours ensemble : changer les portées sans l'espace de jeu reproduit mal le combat.
- Réutiliser la conduite sans strafe, caméra à la souris et visée manuelle. Si les dimensions changent, mettre à jour les limites communes, le décor, la visée/projection et les points d'apparition, plutôt que plusieurs clamps divergents.
- Retour automatique de caméra désactivé dans les valeurs par défaut et les configurations livrées. Le bouton existant reste fonctionnel ; pas de deuxième réglage équivalent.

### Boss

- Poursuite parmi les démolisseurs/sièges occupés et vivants, choix et fréquence conformes à la référence ; les motos ne deviennent pas des cibles de poursuite ordinaires.
- Accumulation de Gathering Speed au fil du combat ; un changement de cible ne réinitialise pas les charges. Le shutdown les retire conformément à la référence.
- Bélier avec portée, cadence, dégâts et vulnérabilité cumulable ; pas une simple perte continue de PV à proximité.
- Décharges de flammes comme canalisation avec ses dégâts périodiques, rayon et interruption, pas uniquement un dégât final après une fausse incantation de trois secondes.
- Barrages et autres attaques du boss correspondant à la référence, avec ciblage, cadence et écoles corrects ; inclure le canon/napalm si présent dans le profil dix joueurs.
- Shutdown puis reprise : bonus de dégâts reçus, immobilisation, arrêt des attaques concernées, purge de vitesse, éjection et remise en état des tourelles selon les phases source. Les tours ne s'arrêtent que si les règles le prévoient.

### Projection, tourelles, surcharge et secours

- Suivre l'occupant humain, pas seulement le véhicule d'origine : embarqué, chargé en catapulte, en vol, sur le boss, au sol, transporté, revenu ou mort. Aucune capacité de véhicule hors du poste approprié.
- Deux tourelles réelles pour dix joueurs, avec PV, cible, dégâts reçus/infligés et état de destruction ; ne pas assimiler deux passagers présents à deux tourelles détruites. Un seul tireur à distance peut attaquer successivement les deux si la référence le permet.
- Sur le boss, remplacer la barre du passager par un bouton **« Attaquer les tourelles »**, accessible par clic et touche 1. Un clic active l'attaque à cadence contrôlée et cible la prochaine tourelle vivante ; pas de nécessité de marteler le bouton ni de sélection manuelle de classe.
- Afficher les PV des tourelles et la progression de surcharge. Aucune progression de DPS du joueur avant son action. Les passagers IA attaquent sans action du joueur, mais avec les mêmes règles de combat.
- Après destruction, enchaîner automatiquement les dispositifs de surcharge requis puis l'éjection selon la référence. Cette automatisation de commande est une **exception volontaire de fidélité**, justifiée par la demande d'un bouton unique, pas une suppression des conditions et délais de la mécanique.
- Vue/caméra, indications et barre d'action suivent le passager humain sur le boss puis pendant le secours ; ne pas le laisser bloqué sur les compétences du démolisseur. Restaurer la barre normale après retour.
- Soins, dégâts sur le passager, immunités de position et mort suivent la référence. Ne pas continuer à DPS après mort/éjection, ni perdre parce que le véhicule d'origine est détruit alors que l'humain est encore vivant ailleurs sans règle explicite. À pied, le joueur garde le contrôle prévu par cette phase ; l'IA moto doit pouvoir réellement le récupérer et le ramener.

### Quatre tours / hard mode

- Appliquer les bonus passifs HM aux PV et aux écoles de dégâts concernées ; mode quatre tours au lancement et après un nouveau démarrage, sans réinitialiser involontairement le choix de difficulté lors d'un simple changement de rôle.
- Thorim : balises, délai annoncé puis frappe ; positions/nombre/fréquence et dégâts de zone sourcés, pas une frappe périodique artificiellement centrée sur le joueur.
- Hodir : balises poursuivantes, arrêt/verrouillage puis impact, gel et dégâts périodiques ; libération par les bonnes attaques de feu. Pas de dégel gratuit après six secondes ; toute durée maximum vient de la source.
- Mimiron : trajet défini et dépôts de feu avec leurs dates d'activation, durée, zones et dégâts. Importer les points du trajet ou une transformation documentée de ceux-ci, pas inventer une nouvelle boucle plus facile.
- Freya : points d'invocation et cadence, types distincts de plantes lorsque la référence les distingue, PV/dégâts/attaques et comportement corrects, interaction avec goudron/feu.

## Présentation visuelle et ressources

Les visuels traduisent l'état du simulateur ; ils ne déclenchent jamais les dégâts. Remplacer les anneaux seuls par des rayons verticaux lisibles : avertissement, verrouillage et impact distincts, éclairs de Thorim, rayon/comète glacée et prison de glace d'Hodir. La largeur visible et le moment de frappe doivent correspondre à la zone réelle. Effets lumineux sans masquer les véhicules ou exiger de distinguer uniquement une couleur.

Pour les plantes, étendre l'import ciblé M2/SKIN/BLP vers GLB/PNG déjà présent afin d'utiliser les modèles locaux des espèces réellement invoquées, avec animations de déplacement/attaque disponibles. Conserver un secours procédural reconnaissable si l'asset est absent ou invalide. L'import reste préalable, en lecture seule ; aucun accès à WoW ou au réseau en partie. Les émetteurs/shaders propriétaires ne sont pas promis pixel pour pixel : reconstituer les effets dans Bevy. Ce rendu n'est pas une copie complète du moteur graphique de WoW.

Les dimensions, silhouettes et animations des ressources importées doivent être vérifiées en situation, pas seulement la réussite du chargement. Pas de clignotement excessif pour renforcer artificiellement les éclairs.

## Surfaces à modifier et compatibilité

- `crates/combat/src/lib.rs` : données/règles, capacités, IA, phases, erreurs et tests headless. Extraire un module de données du combat seulement si cela rend la table sourcée lisible ; pas d'infrastructure générique d'encounters.
- `crates/driving/src/lib.rs` : défaut de vitesse et éventuelles dimensions partagées ; garder les garanties de conduite existantes.
- `crates/client/src/config.rs`, `settings.toml`, `dist/settings.toml` : valeurs 7 et retour caméra Non. Modifier uniquement ces champs demandés dans les réglages présents, préserver les autres préférences ; ne pas réécrire silencieusement les configurations utilisateur externes.
- `crates/client/src/main.rs`, `raid.rs`, `ui.rs`, `ui/wow_hud.rs` : déplacement, caméra, visée, barre contextuelle, PV/maxima, phase et suivi de l'humain, rapports de validation. Vérifier tous les lecteurs de ressources et de l'état du passager.
- `crates/client/src/scene.rs`, `wow.rs`, `models.rs`, `tools/import_wow.py` et tests d'import : dimensions, effets, modèles, animation et secours.
- `assets/wow/manifest.json`, assets convertis et équivalents dans `dist/assets` : provenance et livraison autonome.
- `Tester.ps1`, `Construire.ps1`, scénarios/tests existants, README et guide de livraison : adapter les hypothèses normalisées et documenter les commandes/limites exactes. Ne pas écraser un exécutable en cours d'utilisation ; conserver les anciennes livraisons jusqu'à vérification de la nouvelle.

Changements de comportement attendus : le boss ne meurt plus avec la facilité du prototype ; le gel, les munitions, les erreurs de conduite et les interruptions peuvent mener au wipe. Aucun sauvegarde de progression de raid à migrer actuellement. Préserver les fichiers non liés, les assets de secours et les réglages non demandés. Aucun droit de redistribution des ressources Blizzard n'est accordé par l'import.

## Critères de validation

1. Tests des valeurs/formules de la table : profil 215, maxima, PV HM vs normal, passifs et écoles ; données critiques sourcées ou blocage explicitement levé avant implémentation.
2. Tests temporels à pas fixe des capacités et du GCD/coûts : pas de ressource ou recharge consommée sur une commande invalide, ni de différence gratuite entre IA et humain.
3. Régression boost : démolisseur humain poursuivi, passager IA présent => boost payé et actif ; absent/chargé/mort, ressources insuffisantes, recharge, pas de poursuite => refus selon la règle.
4. Régression projection : atterrissage réussi et manqué, accès au bouton depuis le HUD et le clavier, attaque d'une puis deux tourelles, un seul passager, surcharge/éjection/respawn, retour par moto ; pas de DPS humain automatique sans clic.
5. Tests boss : charges persistantes au changement de poursuite, purge au shutdown, bélier/vulnérabilité, canalisation et interruption, barrages/napalm, reprise et cibles valides.
6. Tests tours : cadence et avertissement/impact, aucune double frappe lors d'un pas traversant le seuil, gel libéré seulement par les bonnes attaques, durée/trajectoire des feux, plantes et effet des écoles/du goudron.
7. Tests configuration : défauts 7/Non, chargement et sauvegarde, conservation des autres préférences, sélection HM initiale ; tests conduite/caméra et projections après changement d'échelle.
8. Scénarios headless longs déterministes, avec budget de PV/dégâts et fin victoire ou wipe, sans invulnérabilité, PV artificiellement réinjectés ni munitions infinies. Comparer les courbes et phases aux valeurs sources, pas à une durée cible choisie. Les essais GPU actuels avec aides restent des tests de rendu, jamais une preuve de difficulté fidèle.
9. Tests Rust de tout le workspace, formatage, tests Python du convertisseur ; essais GPU des cinq postes, du passager sur le boss et récupéré, des quatre tours, des plantes et du secours procédural. Rapports frais, captures et inspection des animations/chronologies.
10. Construire et vérifier une livraison autonome hors de WoW ; confirmer que les modèles/effets proviennent des assets livrés et que les réglages demandés sont effectifs. Ne pas annoncer une fidélité totale si un critère important reste non vérifié.

## Choix laissés à l'implémenteur et points de vigilance

L'implémenteur choisit les structures minimales, le découpage des modules, les maillages/matériaux Bevy et les détails d'ergonomie compatibles avec le bouton unique. Il ne choisit pas arbitrairement une variante historique ou un coefficient de difficulté pour contourner des données absentes. Les valeurs exactes et leurs contradictions seront levées par la table source, préalable au code de combat.

La reproduction du combat reste un entraînement solo : les décisions des neuf autres joueurs sont prises par une IA, non par des humains. Leur timing et leur précision restent un profil d'entraînement documenté ; ils ne justifient ni téléportation, ni visée parfaite cachée, ni garantie de survie. À équipement 215 et règles fidèles, certaines erreurs ou stratégies peuvent légitimement provoquer un wipe.
