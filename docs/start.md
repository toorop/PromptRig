Je veux développer une application desktop cross-platform destinée au prompt engineering et à la comparaison de LLM.

L'application doit rester simple et orientée développeurs. Ce n'est PAS un agent builder, un outil d'observabilité ou une plateforme complexe de workflows. Son rôle principal est de permettre de tester rapidement un prompt sur différents modèles et fournisseurs de LLM.

## Stack technique

Utilise :

- Tauri 2 pour l'application desktop ;
    
- Rust pour le backend Tauri ;
    
- Vue 3 ;
    
- TypeScript ;
    
- Vite ;
    
- shadcn-vue + Tailwind CSS pour l'interface ;
    
- SQLite pour les données persistantes locales ;
    
- le keyring natif du système d'exploitation pour les clés API et autres secrets.
    

L'application doit pouvoir fonctionner sous :

- Linux ;
    
- Windows ;
    
- macOS.
    

Privilégie une architecture simple, lisible et extensible. Évite les abstractions inutiles et l'overengineering.

Je connais bien Vue mais très peu Rust. Le code Rust doit donc être particulièrement clair, idiomatique, découpé en petits modules et correctement commenté lorsqu'une construction Rust n'est pas évidente.

## Objectif du MVP

Je veux essentiellement retrouver l'expérience d'un playground comme celui de Mistral, mais en pouvant choisir librement le fournisseur et le modèle.

L'écran principal doit permettre de :

1. choisir un fournisseur ;
    
2. choisir un modèle disponible chez ce fournisseur ;
    
3. saisir un system prompt ;
    
4. saisir un user prompt ;
    
5. régler les principaux paramètres du modèle ;
    
6. lancer la requête ;
    
7. afficher la réponse ;
    
8. afficher les métriques disponibles concernant cette exécution.
    

Les paramètres doivent au minimum prévoir, lorsqu'ils sont supportés par le modèle :

- temperature ;
    
- max tokens / max completion tokens ;
    
- top_p.
    

Ne force pas l'affichage d'un paramètre qu'un fournisseur ou un modèle ne supporte pas.

## Comparaison côte à côte

Cette fonctionnalité doit être prévue très tôt dans le projet, idéalement juste après le premier vertical slice fonctionnel.

Je veux pouvoir tester exactement le même system prompt et le même user prompt sur plusieurs modèles en parallèle.

L'interface doit permettre de sélectionner plusieurs combinaisons Provider + Model, par exemple :

- Mistral Medium ;
    
- Ministral 8B ;
    
- GPT ;
    
- Claude ;
    
- Gemini.
    

Chaque modèle doit avoir sa propre colonne ou son propre panneau de résultat.

Prévoir un bouton du type :

`Run all`

qui déclenche les différentes requêtes, idéalement en parallèle lorsque cela est pertinent.

Chaque résultat doit afficher indépendamment :

- provider ;
    
- modèle exact ;
    
- texte généré ;
    
- latence ;
    
- nombre de tokens input/output ;
    
- coût estimé ;
    
- paramètres utilisés ;
    
- erreur éventuelle.
    

Le system prompt et le user prompt doivent pouvoir être communs à toutes les colonnes afin de comparer les modèles dans exactement les mêmes conditions.

À terme, il doit également être possible :

- de modifier individuellement le system prompt d'une colonne pour comparer plusieurs variantes ;
    
- de comparer plusieurs paramètres de génération ;
    
- de dupliquer une configuration ;
    
- d'ajouter ou supprimer rapidement une colonne ;
    
- de relancer uniquement une colonne ;
    
- de relancer toute la comparaison ;
    
- de conserver les résultats d'une comparaison.
    

Cette fonctionnalité doit s'appuyer sur les concepts de `Run` et `Experiment` et non introduire une logique parallèle spécifique dans l'interface.

Une comparaison côte à côte correspond conceptuellement à un Experiment contenant plusieurs Runs.

L'objectif est de permettre du prompt engineering comparatif rapide, par exemple pour déterminer quel couple modèle + prompt produit le meilleur résultat en termes de qualité, coût et latence.

## Fournisseurs

L'architecture doit permettre d'ajouter facilement de nouveaux fournisseurs.

Commence par prévoir :

- OpenAI ;
    
- Anthropic ;
    
- Google Gemini ;
    
- Mistral ;
    
- OpenRouter ;
    
- un endpoint générique compatible avec l'API OpenAI.
    

Il doit être possible d'ajouter d'autres providers ultérieurement sans modifier toute l'application.

Définis une abstraction claire côté backend pour les providers.

Par exemple, conceptuellement, un provider doit pouvoir :

- tester sa configuration ;
    
- récupérer ou fournir la liste de ses modèles ;
    
- exposer les capacités principales d'un modèle ;
    
- effectuer une requête de génération ;
    
- retourner une réponse normalisée ;
    
- retourner les informations d'usage lorsqu'elles sont fournies par l'API.
    

Ne cherche pas nécessairement à masquer toutes les différences entre fournisseurs. L'abstraction doit rester pragmatique.

## Gestion des clés API

Les clés API ne doivent jamais être enregistrées en clair dans SQLite, un fichier JSON, localStorage ou le frontend.

Stocke les secrets dans le keyring natif de l'utilisateur :

- macOS Keychain ;
    
- Windows Credential Manager ;
    
- Secret Service sous Linux.
    

Le frontend ne doit jamais avoir besoin de conserver durablement les clés API.

Prévoir un écran Settings / Providers permettant de :

- activer ou désactiver un provider ;
    
- saisir/modifier/supprimer sa clé API ;
    
- éventuellement saisir une base URL personnalisée ;
    
- tester la connexion ;
    
- indiquer visuellement si le provider est correctement configuré.
    

## Interface principale

Je veux une UI desktop sobre, dense et destinée à des développeurs.

Pas d'animations inutiles, pas de design marketing ou bling-bling.

Une première disposition possible :

Barre supérieure :

- Provider ;
    
- Model ;
    
- bouton Run ;
    
- éventuellement quelques informations sur le modèle.
    

Zone principale :

- éditeur System Prompt ;
    
- éditeur User Prompt ;
    
- zone Result.
    

Une disposition en panneaux redimensionnables serait intéressante, sans être indispensable au premier prototype.

Prévoir un accès discret aux paramètres avancés du modèle.

Le résultat doit pouvoir être facilement sélectionné et copié.

Prévoir un thème clair et sombre.

## Résultat d'une exécution

Une exécution doit être représentée explicitement dans le domaine applicatif.

Un Run doit idéalement conserver :

- provider ;
    
- identifiant exact du modèle ;
    
- system prompt ;
    
- user prompt ;
    
- paramètres utilisés ;
    
- résultat texte ;
    
- date/heure ;
    
- durée totale de l'appel ;
    
- éventuellement time-to-first-token si nous implémentons le streaming ;
    
- nombre de tokens input ;
    
- nombre de tokens output ;
    
- coût estimé si l'information tarifaire est disponible ;
    
- erreur éventuelle.
    

Conçois cela de manière à pouvoir ajouter plus tard d'autres métriques.

## Streaming

Prévois l'architecture pour supporter le streaming des réponses.

Il n'est pas obligatoire de terminer le streaming dans la toute première version si cela complexifie fortement le MVP, mais ne construis pas une API interne qui rendrait son ajout difficile.

## Données locales

Utilise SQLite pour stocker les données non sensibles.

Prévois dès maintenant les concepts suivants :

### Prompt

Un prompt peut être nommé et sauvegardé.

Il doit être possible ultérieurement de gérer différentes versions du même prompt.

### Test Case

Un Test Case représente typiquement un user prompt utilisé pour tester un system prompt.

Un même prompt pourra ultérieurement être évalué avec plusieurs Test Cases.

### Run

Un Run est l'exécution d'une combinaison :

Provider + Model + System Prompt + User Prompt + paramètres.

### Experiment

Un Experiment regroupe plusieurs Runs appartenant au même test logique.

Il peut par exemple représenter :

- plusieurs modèles testés avec le même prompt ;
    
- plusieurs variantes de prompt testées avec le même modèle ;
    
- plusieurs paramètres ;
    
- plusieurs Test Cases ;
    
- toute combinaison de ces éléments.
    

La comparaison côte à côte doit utiliser directement cette abstraction.

Ne développe pas nécessairement toute l'interface avancée Experiment dans le MVP, mais prévois ce concept dans l'architecture.

Un Experiment permettra ultérieurement d'exécuter plusieurs combinaisons :

- plusieurs modèles ;
    
- plusieurs variantes de system prompt ;
    
- plusieurs Test Cases ;
    
- éventuellement plusieurs paramètres.
    

Cela permettra de comparer automatiquement les résultats.

## Évolution prévue : optimisation automatique

Cette fonctionnalité n'est PAS à développer dans le MVP, mais l'architecture doit permettre de l'ajouter facilement.

À terme, je veux pouvoir demander à un LLM puissant jouant le rôle de juge/optimiseur de trouver une bonne combinaison entre :

- un system prompt ;
    
- un modèle ;
    
- son coût ;
    
- sa latence ;
    
- la qualité des réponses.
    

Exemple de workflow futur :

1. l'utilisateur décrit le comportement recherché ;
    
2. il fournit plusieurs exemples d'input et éventuellement les outputs attendus ;
    
3. un modèle puissant génère ou améliore un system prompt ;
    
4. l'application teste ce prompt sur différents modèles, éventuellement du plus petit/moins cher au plus puissant ;
    
5. les résultats sont évalués ;
    
6. le prompt est modifié si nécessaire ;
    
7. les tests sont recommencés ;
    
8. l'application propose finalement un couple modèle + prompt avec des métriques de qualité, coût et latence.
    

Il faut donc éviter de coupler la logique actuelle de l'UI à une seule requête interactive.

Les primitives internes doivent permettre d'exécuter des Runs programmatiquement et en série ou en parallèle.

## Architecture souhaitée

Sépare clairement au minimum :

- frontend Vue ;
    
- commandes / API Tauri ;
    
- domaine métier ;
    
- providers LLM ;
    
- stockage SQLite ;
    
- stockage des secrets ;
    
- métriques/coûts.
    

Je veux notamment quelque chose conceptuellement proche de :

providers/  
openai  
anthropic  
mistral  
gemini  
openrouter  
openai_compatible

domain/  
provider  
model  
prompt  
test_case  
run  
experiment

storage/  
database  
secrets

La structure exacte est à déterminer selon les conventions Rust/Tauri actuelles. Ne reproduis pas cette arborescence si une organisation plus idiomatique est préférable.

## Registry de modèles

Ne hardcode pas toute l'application autour d'une liste statique de modèles.

Prévois un Model Registry permettant d'associer à un modèle :

- provider ;
    
- identifiant API ;
    
- nom affiché ;
    
- capacités ;
    
- paramètres supportés ;
    
- éventuellement fenêtre de contexte ;
    
- éventuellement prix input/output ;
    
- autres métadonnées utiles.
    

Lorsque le provider permet de récupérer dynamiquement les modèles, utilise cette possibilité avec un mécanisme de cache.

Certaines métadonnées pourront néanmoins nécessiter un registry local.

## Coût

Prévois dès le départ la notion de coût par Run.

Il n'est pas nécessaire que tous les providers donnent immédiatement un coût exact.

La couche doit pouvoir calculer une estimation à partir :

- des tokens utilisés ;
    
- du prix input ;
    
- du prix output.
    

Le système de pricing doit pouvoir être mis à jour indépendamment du reste de l'application.

## Qualité du code

Je veux utiliser ce projet notamment pour découvrir Rust.

Donc :

- privilégie du Rust idiomatique mais compréhensible ;
    
- évite les macros ou abstractions sophistiquées sans bénéfice réel ;
    
- utilise des types explicites pour le domaine ;
    
- gère proprement les erreurs ;
    
- pas de unwrap() aveugles dans le code de production ;
    
- ajoute des tests unitaires aux parties métier importantes ;
    
- documente les décisions d'architecture importantes.
    

Frontend :

- Composition API ;
    
- `<script setup lang="ts">` ;
    
- composants petits et lisibles ;
    
- état global uniquement lorsqu'il est réellement nécessaire ;
    
- aucun secret persistant côté frontend.
    

## CI/CD

Le dépôt sera hébergé sur GitHub.

Mets en place GitHub Actions.

Je veux au minimum :

### CI sur Pull Request / push

- lint TypeScript ;
    
- typecheck Vue/TypeScript ;
    
- tests frontend ;
    
- cargo fmt --check ;
    
- cargo clippy ;
    
- tests Rust ;
    
- vérification qu'une build Tauri peut être produite.
    

### Releases

Lorsqu'un tag de version est créé, GitHub Actions doit construire les binaires/installateurs pour :

- Linux x86_64 ;
    
- Windows x86_64 ;
    
- macOS Apple Silicon ;
    
- macOS Intel si cela reste raisonnablement supportable.
    

Utilise l'action officielle Tauri lorsque c'est approprié.

Publie les artefacts dans une GitHub Release.

Ne mets évidemment aucune clé LLM personnelle dans les GitHub Secrets nécessaires aux builds.

Prépare la structure de façon à pouvoir ajouter ultérieurement la signature des binaires et l'auto-update.

## Documentation

Crée :

- README.md ;
    
- instructions de développement Linux/Windows/macOS ;
    
- description de l'architecture ;
    
- instructions de build ;
    
- instructions pour ajouter un nouveau provider ;
    
- instructions pour créer une release.
    

## Méthode de travail

Ne commence pas par implémenter toutes les fonctionnalités.

Commence par :

1. analyser cette specification ;
    
2. vérifier les versions et pratiques actuelles des bibliothèques et outils utilisés ;
    
3. proposer une architecture ;
    
4. identifier les décisions qui auraient un impact difficile à modifier plus tard ;
    
5. créer un plan d'implémentation par petites étapes ;
    
6. initialiser le projet ;
    
7. obtenir rapidement un premier vertical slice fonctionnel.
    

Le premier vertical slice doit idéalement permettre :

- de lancer l'application ;
    
- de configurer un seul provider ;
    
- de stocker sa clé de manière sécurisée ;
    
- de sélectionner un modèle ;
    
- de saisir system prompt + user prompt ;
    
- d'envoyer la requête ;
    
- d'afficher la réponse et la latence.
    

Ensuite, implémente rapidement la comparaison côte à côte avec plusieurs modèles avant de poursuivre vers des fonctionnalités plus complexes.

Une fois ce vertical slice et la comparaison multi-modèles propres et fonctionnels, généralise l'architecture aux autres providers.

Ne développe pas prématurément l'optimiseur automatique ni les fonctionnalités avancées d'Experiment.

Avant de prendre une décision structurante ou d'ajouter une grosse dépendance, explique brièvement pourquoi elle est nécessaire.
