#![allow(dead_code)]

use std::io::Read;
use std::mem;

use byteorder::{LittleEndian, ReadBytesExt};

pub const ENTRY_SCENE: i32 = 70;
pub const ENTRY_X: i32 = 19;
pub const ENTRY_Y: i32 = 20;

// CC.MWidth=480;       --主地图宽
const MAIN_WIDTH: usize = 480;
// CC.MHeight=480;      --主地图高
const MAIN_HEIGHT: usize = 480;
//CC.SWidth=64;     --子场景地图大小
pub const SCENE_WIDTH: usize = 64;
// CC.SHeight=64;
pub const SCENE_HEIGHT: usize = 64;
// CC.DNUM=200;       --D*每个场景的事件数
pub const DNUM: usize = 200;

// CONFIG.XSCALE = 18    -- 贴图宽度的一半
pub const XSCALE: usize = 18;
// CONFIG.YSCALE = 9     -- 贴图高度的一半
pub const YSCALE: usize = 9;

// CC.XSCALE=CONFIG.XSCALE;    --贴图一半的宽高
// CC.YSCALE=CONFIG.YSCALE;

/*
CC.TEAM_NUM=6;          --队伍人数
CC.MY_THING_NUM=200      --主角物品数量
 */
const TEAM_NUM: usize = 6;
const MY_THING_NUM: usize = 200;
const ACTION_FRAME: usize = 5;
const SKILL_NUM: usize = 10;
const ITEM_NUM: usize = 4;

const PERSON_SIZE: usize = mem::size_of::<Person>();
const BASE_SIZE: usize = mem::size_of::<Base>();
const SCENE_SIZE: usize = mem::size_of::<Scene>();

#[derive(Debug)]
pub struct Base {
    // CC.Base_S["乘船"] ={0, 0, 2} - - 起始位置(从0开始)，数据类型(0有符号 1无符号，2字符串)，长度
    boat: i16,
    // CC.Base_S["无用"] ={2, 0, 2};
    useless: i16,
    // CC.Base_S["人X"] ={4, 0, 2};
    person_x: i16,
    // CC.Base_S["人Y"] ={6, 0, 2};
    person_y: i16,
    // CC.Base_S["人X1"] ={8, 0, 2};
    person_x1: i16,
    // CC.Base_S["人Y1"] ={10, 0, 2};
    person_x2: i16,
    // CC.Base_S["人方向"] ={12, 0, 2};
    person_dir: i16,
    // CC.Base_S["船X"] ={14, 0, 2};
    boat_x: i16,
    // CC.Base_S["船Y"] ={16, 0, 2};
    boat_y: i16,
    // CC.Base_S["船X1"] ={18, 0, 2};
    boat_x1: i16,
    // CC.Base_S["船Y1"] ={20, 0, 2};
    boat_y1: i16,
    // CC.Base_S["船方向"] ={22, 0, 2};
    boat_dir: i16,
    /*
    for i = 1,
    CC.TEAM_NUM do
    CC.Base_S["队伍"..i]={24 + 2 * (i - 1),0, 2};
    end
     */
    teams: [i16; TEAM_NUM],
    /*
    for i = 1,
    CC.MY_THING_NUM do
    CC.Base_S["物品"..i]={36 + 4 * (i - 1),0, 2};
    CC.Base_S["物品数量"..i]={36 + 4 * (i - 1) +2, 0, 2};
    end
     */
    items: [(i16, i16); MY_THING_NUM],
}

#[derive(Debug)]
pub struct Scene {
    // CC.Scene_S["代号"]={0,0,2}
    code: i16,
    // CC.Scene_S["名称"]={2,2,20}
    name: [u8; 20],
    // CC.Scene_S["出门音乐"]={22,0,2}
    leave_music: i16,
    // CC.Scene_S["进门音乐"]={24,0,2}
    enter_music: i16,
    // CC.Scene_S["跳转场景"]={26,0,2}
    jump: i16,
    // CC.Scene_S["进入条件"]={28,0,2}
    enter_pre: i16,
    // CC.Scene_S["外景入口X1"]={30,0,2}
    out_entry_x1: i16,
    // CC.Scene_S["外景入口Y1"]={32,0,2}
    out_entry_y1: i16,
    // CC.Scene_S["外景入口X2"]={34,0,2}
    out_entry_x2: i16,
    // CC.Scene_S["外景入口Y2"]={36,0,2}
    out_entry_y2: i16,
    // CC.Scene_S["入口X"]={38,0,2}
    entry_x: i16,
    // CC.Scene_S["入口Y"]={40,0,2}
    entry_y: i16,
    // CC.Scene_S["出口X1"]={42,0,2}
    exit_x1: i16,
    // CC.Scene_S["出口X2"]={44,0,2}
    exit_x2: i16,
    // CC.Scene_S["出口X3"]={46,0,2}
    exit_x3: i16,
    // CC.Scene_S["出口Y1"]={48,0,2}
    exit_y1: i16,
    // CC.Scene_S["出口Y2"]={50,0,2}
    exit_y2: i16,
    // CC.Scene_S["出口Y3"]={52,0,2}
    exit_y3: i16,
    // CC.Scene_S["跳转口X1"]={54,0,2}
    jump_x1: i16,
    // CC.Scene_S["跳转口Y1"]={56,0,2}
    jump_y1: i16,
    // CC.Scene_S["跳转口X2"]={58,0,2}
    jump_x2: i16,
    // CC.Scene_S["跳转口Y2"]={60,0,2}
    jump_y2: i16,
}


#[repr(C)]
#[derive(Debug)]
pub struct Person {
    // CC.Person_S["代号"]={0,0,2}
    code: i16,
    // CC.Person_S["头像代号"]={2,0,2}
    avatar: i16,
    // CC.Person_S["生命增长"]={4,0,2}
    life_gain: i16,
    // CC.Person_S["无用"]={6,0,2}
    useless: i16,
    // CC.Person_S["姓名"]={8,2,20}
    name: [u8; 20],
    // CC.Person_S["外号"]={28,2,20}
    alias: [u8; 20],
    // CC.Person_S["性别"]={48,0,2}
    male: i16,
    // CC.Person_S["等级"]={50,0,2}
    level: i16,
    // CC.Person_S["经验"]={52,1,2}
    exp: u16,
    // CC.Person_S["生命"]={54,0,2}
    life: i16,
    // CC.Person_S["生命最大值"]={56,0,2}
    life_max: i16,
    // CC.Person_S["受伤程度"]={58,0,2}
    injure: i16,
    // CC.Person_S["中毒程度"]={60,0,2}
    tox: i16,
    // CC.Person_S["体力"]={62,0,2}
    vatity: i16,
    // CC.Person_S["物品修炼点数"]={64,0,2}
    item_familiar: i16,
    // CC.Person_S["武器"]={66,0,2}
    weapon: i16,
    // CC.Person_S["防具"]={68,0,2}
    armor: i16,
    // actions
    // for i=1,5 do
    // CC.Person_S["出招动画帧数" .. i]={70+2*(i-1),0,2};
    // CC.Person_S["出招动画延迟" .. i]={80+2*(i-1),0,2};
    // CC.Person_S["武功音效延迟" .. i]={90+2*(i-1),0,2};
    // end
    action_frames: [i16; ACTION_FRAME],
    action_delays: [i16; ACTION_FRAME],
    action_audio_delays: [i16; ACTION_FRAME],
    // CC.Person_S["内力性质"]={100,0,2}
    neili_status: i16,
    // CC.Person_S["内力"]={102,0,2}
    neili: i16,
    // CC.Person_S["内力最大值"]={104,0,2}
    neili_max: i16,
    // CC.Person_S["攻击力"] ={106, 0, 2}
    attack: i16,
    // CC.Person_S["轻功"] ={108, 0, 2}
    agile: i16,
    // CC.Person_S["防御力"] ={110, 0, 2}
    defence: i16,
    // CC.Person_S["医疗能力"] ={112, 0, 2}
    cure: i16,
    // CC.Person_S["用毒能力"] ={114, 0, 2}
    poison: i16,
    // CC.Person_S["解毒能力"] ={116, 0, 2}
    depoison: i16,
    // CC.Person_S["抗毒能力"] ={118, 0, 2}
    poison_def: i16,
    // CC.Person_S["拳掌功夫"] ={120, 0, 2}
    fist: i16,
    // CC.Person_S["御剑能力"] ={122, 0, 2}
    sword: i16,
    // CC.Person_S["耍刀技巧"] ={124, 0, 2}
    knife: i16,
    // CC.Person_S["特殊兵器"] ={126, 0, 2}
    other_weapon: i16,
    // CC.Person_S["暗器技巧"] ={128, 0, 2}
    fly_weapon: i16,
    // CC.Person_S["武学常识"] ={130, 0, 2}
    wknowlege: i16,
    // CC.Person_S["品德"] ={132, 0, 2}
    sanity: i16,
    // CC.Person_S["攻击带毒"] ={134, 0, 2}
    with_poison: i16,
    // CC.Person_S["左右互搏"] ={136, 0, 2}
    double_att: i16,
    // CC.Person_S["声望"] ={138, 0, 2}
    reputation: i16,
    // CC.Person_S["资质"] ={140, 0, 2}
    pub talent: i16,
    // CC.Person_S["修炼物品"] ={142, 0, 2}
    item_train: i16,
    // CC.Person_S["修炼点数"] ={144, 0, 2}
    item_point: i16,
    /*
    for i = 1,
    10 do
    CC.Person_S["武功"..i]={146 + 2 * (i - 1),0, 2};
    CC.Person_S["武功等级"..i]={166 + 2 * (i - 1),0, 2};
    end
     */
    skills: [i16; SKILL_NUM],
    skill_levels: [i16; SKILL_NUM],
    /*
    for i = 1,
    4 do
    CC.Person_S["携带物品"..i]={186 + 2 * (i - 1),0, 2};
    CC.Person_S["携带物品数量"..i]={194 + 2 * (i - 1),0, 2};
    end
     */
    items: [i16; ITEM_NUM],
    item_nums: [i16; ITEM_NUM],
}

fn to_str(v: &[u8]) -> String {
    let out = v.iter().map_while(|v| {
        if *v == 0 {
            None
        } else {
            Some(*v)
        }
    }).collect::<Vec<u8>>();
    String::from_utf8(out).unwrap()
}

macro_rules! read {
    ($ident : ident, i16) => {
        $ident.read_i16::<LittleEndian>().unwrap()
    };
    ($ident : ident, u16) => {
        $ident.read_u16::<LittleEndian>().unwrap()
    };
    ($ident : ident, $val : ident, u8) => {
        $ident.read($vall);
    };
}


impl Person {
    pub fn new(data: &[u8]) -> Self {
        let mut c = std::io::Cursor::new(data);
        let code = read!(c, i16);
        let avatar = read!(c, i16);
        let life_gain = read!(c, i16);
        let useless = read!(c, i16);
        let mut name = [0; 20];
        c.read(&mut name).unwrap();
        let mut alias = [0; 20];
        c.read(&mut alias).unwrap();
        let male = read!(c, i16);
        let level = read!(c, i16);
        let exp = read!(c, u16);
        let life = read!(c, i16);
        let life_max = read!(c, i16);
        let injure = read!(c, i16);
        let tox = read!(c, i16);
        let vatity = read!(c, i16);
        let item_familiar = read!(c, i16);
        let weapon = read!(c, i16);
        let armor = read!(c, i16);
        let action_frames = [0; ACTION_FRAME].map(|_| read!(c, i16));
        let action_delays = [0; ACTION_FRAME].map(|_| read!(c, i16));
        let action_audio_delays = [0; ACTION_FRAME].map(|_| read!(c, i16));
        let neili_status = read!(c, i16);
        let neili = read!(c, i16);
        let neili_max = read!(c, i16);
        let attack = read!(c, i16);
        let agile = read!(c, i16);
        let defence = read!(c, i16);
        let cure = read!(c, i16);
        let poison = read!(c, i16);
        let depoison = read!(c, i16);
        let poison_def = read!(c, i16);
        let fist = read!(c, i16);
        let sword = read!(c, i16);
        let knife = read!(c, i16);
        let other_weapon = read!(c, i16);
        let fly_weapon = read!(c, i16);
        let wknowlege = read!(c, i16);
        let sanity = read!(c, i16);
        let with_poison = read!(c, i16);
        let double_att = read!(c, i16);
        let reputation = read!(c, i16);
        let talent = read!(c, i16);
        let item_train = read!(c, i16);
        let item_point = read!(c, i16);
        let skills = [0; SKILL_NUM].map(|_| read!(c, i16));
        let skill_levels = [0; SKILL_NUM].map(|_| read!(c, i16));
        let items = [0; ITEM_NUM].map(|_| read!(c, i16));
        let item_nums = [0; ITEM_NUM].map(|_| read!(c, i16));
        Person {
            code,
            avatar,
            life_gain,
            useless,
            name,
            alias,
            male,
            level,
            exp,
            life,
            life_max,
            injure,
            tox,
            vatity,
            item_familiar,
            weapon,
            armor,
            action_frames,
            action_delays,
            action_audio_delays,
            neili_status,
            neili,
            neili_max,
            attack,
            agile,
            defence,
            cure,
            poison,
            depoison,
            poison_def,
            fist,
            sword,
            knife,
            other_weapon,
            fly_weapon,
            wknowlege,
            sanity,
            with_poison,
            double_att,
            reputation,
            talent,
            item_train,
            item_point,
            skills,
            skill_levels,
            items,
            item_nums,
        }
    }

    pub fn name(&self) -> String {
        to_str(&self.name)
    }

    pub fn alias(&self) -> String {
        to_str(&self.alias)
    }
}

impl Base {
    pub fn new(data: &[u8]) -> Self {
        let mut c = std::io::Cursor::new(data);
        Base {
            boat: read!(c, i16),
            useless: read!(c, i16),
            person_x: read!(c,i16),
            person_y: read!(c, i16),
            person_x1: read!(c,i16),
            person_x2: read!(c,i16),
            person_dir: read!(c,i16),
            boat_x: read!(c,i16),
            boat_y: read!(c,i16),
            boat_x1: read!(c,i16),
            boat_y1: read!(c, i16),
            boat_dir: read!(c, i16),
            teams: [0; TEAM_NUM].map(|_| {
                read!(c,i16)
            }),
            items: [0; MY_THING_NUM].map(|_| {
                (read!(c,i16), read!(c, i16))
            }),
        }
    }
}

impl Scene {
    pub fn new(data: &[u8]) -> Self {
        let mut c = std::io::Cursor::new(data);
        Scene {
            code: read!(c, i16),
            name: {
                let mut name = [0; 20];
                c.read(&mut name).unwrap();
                name
            },
            leave_music: read!(c, i16),
            enter_music: read!(c, i16),
            jump: read!(c, i16),
            enter_pre: read!(c, i16),
            out_entry_x1: read!(c, i16),
            out_entry_y1: read!(c, i16),
            out_entry_x2: read!(c, i16),
            out_entry_y2: read!(c, i16),
            entry_x: read!(c, i16),
            entry_y: read!(c, i16),
            exit_x1: read!(c, i16),
            exit_x2: read!(c, i16),
            exit_x3: read!(c, i16),
            exit_y1: read!(c, i16),
            exit_y2: read!(c, i16),
            exit_y3: read!(c, i16),
            jump_x1: read!(c, i16),
            jump_y1: read!(c, i16),
            jump_x2: read!(c, i16),
            jump_y2: read!(c, i16),
        }
    }

    pub fn name(&self) -> String {
        to_str(&self.name)
    }
}