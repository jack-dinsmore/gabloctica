import numpy as np
from scipy.spatial.transform import Rotation

all_rotations = []
# Facing forward
for angle in range(4):
    rotation = Rotation.from_euler("zyz", (np.pi/2, np.pi/2, angle*np.pi/2))
    all_rotations.append(rotation)
# Facing backward
for angle in range(4):
    rotation = Rotation.from_euler("zyz", (-np.pi/2, np.pi/2, angle*np.pi/2))
    all_rotations.append(rotation)
# Facing left
for angle in range(4):
    rotation = Rotation.from_euler("zyz", (0, np.pi/2, angle*np.pi/2))
    all_rotations.append(rotation)
# Facing right
for angle in range(4):
    rotation = Rotation.from_euler("zyz", (np.pi, np.pi/2, angle*np.pi/2))
    all_rotations.append(rotation)
# Facing up
for angle in range(4):
    rotation = Rotation.from_euler("zyz", (0, 0, angle*np.pi/2))
    all_rotations.append(rotation)
# Facing down
for angle in range(4):
    rotation = Rotation.from_euler("zyz", (0, np.pi, angle*np.pi/2))
    all_rotations.append(rotation)
all_rotations = np.array(all_rotations)

def get_index(mvec, vec):
    deltas = [
        ([4, 1, 1, 1], [0,0]),
        ([4, -1, 1, 1], [1,0]),
        ([4, 1, -1, 1], [0,1]),
        ([4, -1, -1, 1], [1,1]),

        ([5, 1, 1, -1], [0,1]),
        ([5, -1, 1, -1], [1,1]),
        ([5, 1, -1, -1], [0,2]),
        ([5, -1, -1, -1], [1,2]),

        ([2, 1, 1, 1], [0,2]),
        ([2, -1, 1, 1], [1,2]),
        ([2, 1, 1, -1], [0,3]),
        ([2, -1, 1, -1], [1,3]),

        ([3, 1, -1, 1], [0,3]),
        ([3, -1, -1, 1], [1,3]),
        ([3, 1, -1, -1], [0,4]),
        ([3, -1, -1, -1], [1,4]),

        ([0, 1, 1, 1], [0,4]),
        ([0, 1, -1, 1], [1,4]),
        ([0, 1, 1, -1], [0,5]),
        ([0, 1, -1, -1], [1,5]),

        ([1, -1, 1, 1], [0,5]),
        ([1, -1, -1, 1], [1,5]),
        ([1, -1, 1, -1], [0,6]),
        ([1, -1, -1, -1], [1,6]),
    ]
    outputs = []
    for rotation in all_rotations:
        inv_mat = np.transpose(rotation.as_matrix())
        new_mvec = inv_mat @ mvec
        linf_norm = np.max(np.abs(new_mvec))
        if linf_norm == new_mvec[0]:
            face = 0
        if linf_norm == -new_mvec[0]:
            face = 1
        if linf_norm == new_mvec[1]:
            face = 2
        if linf_norm == -new_mvec[1]:
            face = 3
        if linf_norm == new_mvec[2]:
            face = 4
        if linf_norm == -new_mvec[2]:
            face = 5

        original_vec = inv_mat @ vec
        original_vec = np.concatenate([[face], original_vec])
        for v, d in deltas:
            if np.sum(np.abs(np.array(v) - original_vec)) < 0.001:
                outputs.append(d)
                break
    return np.array(outputs)

def print_code(vecs):
    outputs = []
    mvec = np.mean(vecs, axis=0)
    for vec in vecs:
        outputs.append(get_index(mvec, vec))
    print("match self.ori {")
    for i in range(len(all_rotations)):
        print(f"\t{i} => ( "\
            f"((self.id as u32 + {outputs[0][i][0]})<<16) | ({outputs[0][i][1]}<<20), "\
            f"((self.id as u32 + {outputs[1][i][0]})<<16) | ({outputs[1][i][1]}<<20), "\
            f"((self.id as u32 + {outputs[2][i][0]})<<16) | ({outputs[2][i][1]}<<20), "\
            f"((self.id as u32 + {outputs[3][i][0]})<<16) | ({outputs[3][i][1]}<<20) ),")
    print("\t_ => unreachable!()")
    print("}")

# forward
print_code([[1,-1,-1], [1,1,-1], [1,1,1], [1,-1,1]])
# backward
# print_code([[-1,-1,1], [-1,1,1], [-1,1,-1], [-1,-1,-1]])
# # left
# print_code([[-1,1,-1], [1,1,-1], [1,1,1], [-1,1,1]])
# # right
# print_code([[-1,-1,-1], [1,-1,-1], [1,-1,1], [-1,-1,1]])
# # up
# print_code([[-1,1,1], [1,1,1], [1,-1,1], [-1,-1,1]])
# # down
# print_code([[-1,-1,-1], [1,-1,-1], [1,1,-1], [-1,1,-1]])