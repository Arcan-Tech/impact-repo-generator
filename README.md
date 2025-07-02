# Synthetic repository generation for co-change analysis

This repository contains a tool that generates synthetic Git repositories.

The files contained in the repository change following a Markov chain model where every random walk starting from the initial state and ending in the commit state results in a commit in the repository.

## States
The types of states in a model are the following:

- **Initial**: a meta state denoting the starting position of the Markov chain;
- **Issue**: a state denoting a logical change to a specific set of files that may span multiple commits;
- **Module**: a meta state that groups files together and makes it easier describing the model;
- **File**: a state denoting a single file that, if included in the path, it will be committed; *Must not contain folders*, only the file name (ie. file1.txt);
- **Author**: a state denoting the author of the commitj;
- **Commit**: a meta state denoting the ending position of the random walk in the markov chain;


## Transitions
A Markov chain changes state from `s1` to `s2` following a certain user-defined probability `p_{1, 2}`. 
Every state `s` can transit to any other state with a given probability. To select such probability, we draw from a uniform random distribution and compare the value with the ranked transition probabilities. The closest and largest transition probability is the chosen transition (check `src/generator/state.rs:194`);

Given a Model, by following a path from **Initial** to **Commit** we can generate a commits that change files based on a specific a priori probability.
All paths in the simulation should land at least once in every state (except **Module**, which is optional).
In order for paths to be valid, the following conditions must hold:

1. all outgoing transition probabilities of a given state `s` should sum up to 1;
2. at least one **Commit** state should be defined;
3. **Commit** states should have no outgoing transitions;
4. at least one **Author** should be defined;
5. at least one **File** should be defined (not enforced).

Termination of a simulation is not guaranteed.

## Issue selection
The transition from the **Initial** state to an **Issue** state is a special case, as one may want to have a run sequence of certain issues (e.g. `#42`) to simulate a developer working on a specific issue but making multiple commits.

This behavior can be simulated with the dedicated `issue_sequence.average_consecutive_commits` field in the `markov.yaml` file.
In this case, the transition does not abide only by the transition probability, but also by an Exponential probability that decides whehter to repeat the previous issue, or draw another one (with repetition).

## Issue naming
The naming of the issues in the commit messages follows a specific naming convention: `<name_of_issue_state>_<commit number>`.
This format allows to generate multiple issues while knowing the original meta issue they originate from.
For example, `issue1_1` and `issue1_2` are two different issue that both originate from `issue1` **Issue** state.

# Usage 
Running
```sh
./repo-generator generate -m ./test_data/markov.yaml -r /tmp/output_repo -c 100 --hours 48 --start 2023-01-01T12:00:00
```
will generate a repository at `/tmp/output_repo` with `-c 100` commits with an average of 48 hours interval between them, starting from January 1 2023.

Running
```sh
./repo-generator dot -m ./test_data/markov.yaml -o /tmp/markov.dot
```
giving the markov model as input will generate the    corresponding .dot file at `/tmp/markov.dot`.

Use the command graphviz `dot -Tpng markov.dot -o markov.png` to generate the png of the graph.


# Model file

Example `markov.yaml`:
```yaml
issue_sequence:
  average_consecutive_commits:
   !Issue issue1: 10.0 # Average of 10 commits denoted with issue1
   !Issue issue2: 2.0

transitions:
  matrix:
    !Initial : 
    # Transition from Initial state to issue1 has 0.7 likelihood of happening
      - to: !Issue issue1
        p: 0.7

    # Transition from Initial state to issue2 has 0.3 likelihood of happening
      - to: !Issue issue2
        p: 0.3

    !Issue issue1:
      - to: !Module m1
        p: 1.0

    !Issue issue2:
      - to: !Module m1
        p: 1.0

    !File file1:
      - to: !File file2
        p: 0.32
      - to: !File file3
        p: 0.10
      - to: !Author author1
        p: 0.58

    !File file2:
      - to: !File file3
        p: 0.80
      - to: !File file1
        p: 0.10
      - to: !Author author1
        p: 0.10

    !File file3:
      - to: !File file2
        p: 0.85
      - to: !File file1
        p: 0.10
      - to: !Author author1
        p: 0.05

    !Module m1:
      - to: !File file1
        p: 1

    !Author author1:
      - to: !Commit
        p: 1
```

