//! M9 词书与种子词库。
//!
//! 词书按学段 / 考试划分（小学、初中、高中、大学、雅思、专业），每个词书带一个
//! 难度系数（difficulty）：难度越高，SRS 首次复习间隔越短（更频繁巩固）。
//! 每个单词还可带内在难度（intrinsic difficulty，默认 1.0）做细粒度调整；
//! 学习新词时有效难度 = 词书难度 × 单词内在难度，快照写入 `word_states.difficulty`。
//! 迁移 1 已存在的 20 个种子单词会按难度映射进对应词书，保持数据唯一约束。

pub struct SeedWord {
    pub word: &'static str,
    pub phonetic: &'static str,
    pub meaning: &'static str,
    pub example_en: &'static str,
    pub example_zh: &'static str,
    pub difficulty: f64,
}

pub struct SeedBook {
    pub code: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub level: &'static str,
    pub difficulty: f64,
    pub sort_order: i64,
    pub words: &'static [SeedWord],
}

/// 内在难度 1.0（常规单词）。
const fn w(
    word: &'static str,
    phonetic: &'static str,
    meaning: &'static str,
    example_en: &'static str,
    example_zh: &'static str,
) -> SeedWord {
    SeedWord {
        word,
        phonetic,
        meaning,
        example_en,
        example_zh,
        difficulty: 1.0,
    }
}

/// 内在难度 1.25（较难单词）。
const fn hw(
    word: &'static str,
    phonetic: &'static str,
    meaning: &'static str,
    example_en: &'static str,
    example_zh: &'static str,
) -> SeedWord {
    SeedWord {
        word,
        phonetic,
        meaning,
        example_en,
        example_zh,
        difficulty: 1.25,
    }
}

/// 内在难度 1.5（专业长词等最难单词）。
const fn vw(
    word: &'static str,
    phonetic: &'static str,
    meaning: &'static str,
    example_en: &'static str,
    example_zh: &'static str,
) -> SeedWord {
    SeedWord {
        word,
        phonetic,
        meaning,
        example_en,
        example_zh,
        difficulty: 1.5,
    }
}

const PRIMARY_WORDS: &[SeedWord] = &[
    w("apple", "/ˈæpl/", "苹果", "I eat an apple every day.", "我每天吃一个苹果。"),
    w("banana", "/bəˈnɑːnə/", "香蕉", "The monkey likes bananas.", "猴子喜欢香蕉。"),
    w("cat", "/kæt/", "猫", "The cat is sleeping on the sofa.", "猫在沙发上睡觉。"),
    w("dog", "/dɒɡ/", "狗", "My dog likes to run in the park.", "我的狗喜欢在公园里跑。"),
    w("egg", "/eɡ/", "鸡蛋", "We have eggs for breakfast.", "我们早餐吃鸡蛋。"),
    w("fish", "/fɪʃ/", "鱼", "Fish can swim very fast.", "鱼游得很快。"),
    w("girl", "/ɡɜːl/", "女孩", "The girl is reading a book.", "那个女孩正在读书。"),
    w("house", "/haʊs/", "房子", "Our house has a small garden.", "我们的房子有一个小花园。"),
    w("ice", "/aɪs/", "冰", "Ice melts in the sun.", "冰在阳光下融化。"),
    w("juice", "/dʒuːs/", "果汁", "I drink orange juice in the morning.", "我早上喝橙汁。"),
    w("kite", "/kaɪt/", "风筝", "The boy flies a kite in the wind.", "男孩在风中放风筝。"),
    w("lion", "/ˈlaɪən/", "狮子", "The lion is the king of the jungle.", "狮子是丛林之王。"),
    w("milk", "/mɪlk/", "牛奶", "Milk is good for your bones.", "牛奶对你的骨骼有好处。"),
    w("nose", "/nəʊz/", "鼻子", "Your nose helps you smell flowers.", "鼻子帮你闻花香。"),
    w("orange", "/ˈɒrɪndʒ/", "橙子；橙色", "The orange is sweet and juicy.", "这个橙子又甜又多汁。"),
    w("pen", "/pen/", "钢笔", "May I borrow your pen?", "我可以借你的钢笔吗？"),
    w("queen", "/kwiːn/", "女王", "The queen lives in a big palace.", "女王住在一座大宫殿里。"),
    w("rain", "/reɪn/", "雨", "The rain makes the grass grow.", "雨水让草生长。"),
    w("sun", "/sʌn/", "太阳", "The sun rises in the east.", "太阳从东方升起。"),
    w("tree", "/triː/", "树", "Birds build nests in the tree.", "鸟儿在树上筑巢。"),
    w("umbrella", "/ʌmˈbrelə/", "雨伞", "Take an umbrella when it rains.", "下雨时要带伞。"),
    w("violin", "/ˌvaɪəˈlɪn/", "小提琴", "She plays the violin beautifully.", "她小提琴拉得很好听。"),
    w("water", "/ˈwɔːtə/", "水", "Please drink more water every day.", "请每天多喝水。"),
    w("yellow", "/ˈjeləʊ/", "黄色", "The yellow flowers are beautiful.", "黄色的花很漂亮。"),
    w("zoo", "/zuː/", "动物园", "We saw pandas at the zoo.", "我们在动物园看到了熊猫。"),
    w("friend", "/frend/", "朋友", "A good friend is always kind.", "好朋友总是很友善。"),
    w("family", "/ˈfæməli/", "家庭", "My family has four people.", "我家有四口人。"),
    w("teacher", "/ˈtiːtʃə/", "老师", "Our teacher explains everything clearly.", "我们的老师把一切都讲得很清楚。"),
];

const JUNIOR_WORDS: &[SeedWord] = &[
    w("believe", "/bɪˈliːv/", "相信", "I believe you can do it.", "我相信你能做到。"),
    w("borrow", "/ˈbɒrəʊ/", "借入", "Can I borrow your dictionary?", "我可以借你的字典吗？"),
    w("century", "/ˈsentʃəri/", "世纪", "The museum was built in the 19th century.", "这座博物馆建于 19 世纪。"),
    w("dictionary", "/ˈdɪkʃənəri/", "词典", "Look up the word in the dictionary.", "在词典里查一下这个词。"),
    w("environment", "/ɪnˈvaɪrənmənt/", "环境", "We should protect the environment.", "我们应该保护环境。"),
    w("favorite", "/ˈfeɪvərɪt/", "最喜爱的", "Swimming is my favorite sport.", "游泳是我最喜爱的运动。"),
    w("geography", "/dʒiˈɒɡrəfi/", "地理", "Geography helps us understand the world.", "地理帮助我们了解世界。"),
    w("hospital", "/ˈhɒspɪtl/", "医院", "She works at the city hospital.", "她在市医院工作。"),
    w("invite", "/ɪnˈvaɪt/", "邀请", "Let's invite Tom to the party.", "我们邀请汤姆来参加聚会吧。"),
    w("journey", "/ˈdʒɜːni/", "旅程", "The journey took three hours by train.", "这段旅程坐火车花了三个小时。"),
    w("knowledge", "/ˈnɒlɪdʒ/", "知识", "Knowledge is power.", "知识就是力量。"),
    w("library", "/ˈlaɪbrəri/", "图书馆", "I study in the library after class.", "课后我在图书馆学习。"),
    w("memory", "/ˈmeməri/", "记忆；记忆力", "Exercise can improve your memory.", "锻炼能提高记忆力。"),
    w("neighbor", "/ˈneɪbə/", "邻居", "Our neighbor is very friendly.", "我们的邻居很友好。"),
    w("opportunity", "/ˌɒpəˈtjuːnəti/", "机会", "Don't miss this good opportunity.", "不要错过这个好机会。"),
    w("practice", "/ˈpræktɪs/", "练习", "Practice makes perfect.", "熟能生巧。"),
    w("quiet", "/ˈkwaɪət/", "安静的", "Please keep quiet in the library.", "在图书馆请保持安静。"),
    w("respect", "/rɪˈspekt/", "尊重", "We should respect different opinions.", "我们应该尊重不同的意见。"),
    w("science", "/ˈsaɪəns/", "科学", "Science explains how things work.", "科学解释事物如何运作。"),
    w("travel", "/ˈtrævl/", "旅行", "I want to travel around the world.", "我想环游世界。"),
    w("useful", "/ˈjuːsfəl/", "有用的", "This map is very useful for tourists.", "这张地图对游客很有用。"),
    w("volunteer", "/ˌvɒlənˈtɪə/", "志愿者；自愿做", "Many students volunteer at the shelter.", "许多学生在收容所做志愿者。"),
    w("weather", "/ˈweðə/", "天气", "The weather is sunny today.", "今天天气晴朗。"),
    w("yesterday", "/ˈjestədeɪ/", "昨天", "Yesterday we visited the science museum.", "昨天我们参观了科学博物馆。"),
    w("advice", "/ədˈvaɪs/", "建议", "Thank you for your useful advice.", "谢谢你有用的建议。"),
    w("experience", "/ɪkˈspɪəriəns/", "经验；经历", "She has rich teaching experience.", "她有丰富的教学经验。"),
    w("habit", "/ˈhæbɪt/", "习惯", "Reading is a good habit.", "阅读是一个好习惯。"),
    w("success", "/səkˈses/", "成功", "Hard work leads to success.", "努力带来成功。"),
];

const SENIOR_WORDS: &[SeedWord] = &[
    w("abundant", "/əˈbʌndənt/", "丰富的；充足的", "The region has abundant natural resources.", "该地区自然资源丰富。"),
    w("circumstance", "/ˈsɜːkəmstəns/", "情况；环境", "Under these circumstances, we have to wait.", "在这种情况下，我们只能等待。"),
    hw("dilemma", "/dɪˈlemə/", "困境；进退两难", "She faced a dilemma between work and family.", "她在工作和家庭之间进退两难。"),
    w("endeavor", "/ɪnˈdevə/", "努力", "We must endeavor to improve our skills.", "我们必须努力提升技能。"),
    w("genuine", "/ˈdʒenjuɪn/", "真诚的；真正的", "He showed genuine interest in the project.", "他对这个项目表现出真诚的兴趣。"),
    w("hierarchy", "/ˈhaɪərɑːki/", "等级制度", "The company has a clear hierarchy.", "这家公司有清晰的等级制度。"),
    w("inevitable", "/ɪnˈevɪtəbl/", "不可避免的", "Change is inevitable in any growing city.", "任何成长中的城市都不可避免变化。"),
    w("legitimate", "/lɪˈdʒɪtɪmət/", "合法的；正当的", "Is that a legitimate reason for leaving early?", "那是提前离开的正当理由吗？"),
    w("momentum", "/məˈmentəm/", "势头；动量", "The campaign is gaining momentum.", "这场运动势头正盛。"),
    w("negotiate", "/nɪˈɡəʊʃieɪt/", "谈判；协商", "We need to negotiate a better deadline.", "我们需要协商一个更好的截止时间。"),
    w("obscure", "/əbˈskjʊə/", "模糊的；鲜为人知的", "The meaning of this poem is obscure.", "这首诗的含义很晦涩。"),
    w("paramount", "/ˈpærəmaʊnt/", "至为重要的", "Safety is paramount in this industry.", "在这个行业，安全至关重要。"),
    w("quantify", "/ˈkwɒntɪfaɪ/", "量化", "It is hard to quantify the value of health.", "健康的价值很难量化。"),
    w("resilient", "/rɪˈzɪliənt/", "有韧性的", "Children are often more resilient than adults expect.", "孩子往往比大人想象中更有韧性。"),
    w("sophisticated", "/səˈfɪstɪkeɪtɪd/", "复杂的；精密的", "This is a sophisticated piece of software.", "这是一个精密的软件。"),
    w("transparent", "/trænsˈpærənt/", "透明的", "The process should be transparent and fair.", "流程应当透明公平。"),
    w("unanimous", "/juˈnænɪməs/", "一致同意的", "The board reached a unanimous decision.", "董事会达成了一致决定。"),
    w("viable", "/ˈvaɪəbl/", "可行的；可存活的", "This plan is not economically viable.", "这个计划在经济上不可行。"),
    w("withstand", "/wɪðˈstænd/", "承受；抵挡", "The bridge can withstand strong winds.", "这座桥能承受强风。"),
    w("yield", "/jiːld/", "产生；屈服", "The research yielded surprising results.", "研究产生了令人惊讶的结果。"),
    hw("zenith", "/ˈzenɪθ/", "顶峰；最高点", "The empire was at its zenith in the 15th century.", "该帝国在 15 世纪达到鼎盛。"),
    w("feasible", "/ˈfiːzəbl/", "可行的", "A full redesign is not feasible this quarter.", "本季度全面改版不可行。"),
    w("sustain", "/səˈsteɪn/", "维持；支撑", "Small habits can sustain long-term progress.", "小习惯可以维持长期进步。"),
    w("acknowledge", "/əkˈnɒlɪdʒ/", "承认；致谢", "She acknowledged the effort behind the report.", "她认可了这份报告背后的付出。"),
    w("concise", "/kənˈsaɪs/", "简洁的", "Keep your summary concise and clear.", "让你的总结简洁清晰。"),
    w("generate", "/ˈdʒenəreɪt/", "产生；生成", "This exercise will generate new ideas.", "这个练习会产生新想法。"),
    w("perspective", "/pəˈspektɪv/", "视角；观点", "Travel gives you a new perspective on life.", "旅行会给你看待生活的新视角。"),
    w("subtle", "/ˈsʌtl/", "微妙的；不易察觉的", "There is a subtle difference between the two words.", "这两个词之间有微妙的差别。"),
];

const COLLEGE_WORDS: &[SeedWord] = &[
    w("abstract", "/ˈæbstrækt/", "抽象的", "The theory is too abstract for beginners.", "这个理论对初学者来说太抽象了。"),
    w("ambiguous", "/æmˈbɪɡjuəs/", "模棱两可的", "The ending is deliberately ambiguous.", "这个结尾故意留白。"),
    w("coherent", "/kəʊˈhɪərənt/", "连贯的", "The essay needs a more coherent structure.", "这篇文章需要更连贯的结构。"),
    w("criterion", "/kraɪˈtɪəriən/", "标准", "Price is not the only criterion.", "价格不是唯一的标准。"),
    w("empirical", "/ɪmˈpɪrɪkl/", "经验主义的；实证的", "The theory needs empirical support.", "这个理论需要实证支持。"),
    w("framework", "/ˈfreɪmwɜːk/", "框架", "We built a framework for future research.", "我们为未来研究搭建了框架。"),
    hw("heuristic", "/hjuˈrɪstɪk/", "启发式的", "The heuristic method saves time.", "启发式方法节省时间。"),
    w("ideology", "/ˌaɪdiˈɒlədʒi/", "意识形态", "Ideology shapes the way we think.", "意识形态塑造我们的思维方式。"),
    hw("paradigm", "/ˈpærədaɪm/", "范式", "The discovery created a new scientific paradigm.", "这一发现开创了新的科学范式。"),
    w("pragmatic", "/præɡˈmætɪk/", "务实的", "We need a pragmatic approach to this problem.", "我们需要务实地处理这个问题。"),
    w("rigorous", "/ˈrɪɡərəs/", "严格的；严谨的", "The experiment followed rigorous standards.", "实验遵循严格的标准。"),
    hw("semantics", "/sɪˈmæntɪks/", "语义学", "The semantics of this term are debated.", "这个术语的语义有争议。"),
    hw("taxonomy", "/tækˈsɒnəmi/", "分类学", "Biologists use a taxonomy to classify species.", "生物学家用分类法给物种分类。"),
    w("utilitarian", "/ˌjuːtɪlɪˈteəriən/", "功利主义的；实用的", "The design is purely utilitarian.", "这个设计纯粹是实用主义的。"),
    w("verify", "/ˈverɪfaɪ/", "核实", "Please verify the data before publishing.", "发表前请核实数据。"),
    w("threshold", "/ˈθreʃhəʊld/", "门槛；临界点", "We are approaching the threshold of a new era.", "我们正接近一个新时代的门槛。"),
    w("deliberate", "/dɪˈlɪbərət/", "深思熟虑的；刻意的", "She made a deliberate effort to speak slowly.", "她刻意放慢语速。"),
    w("nuance", "/ˈnjuːɑːns/", "细微差别", "The writer captures every nuance of the dialogue.", "作者捕捉到了对话中的每个细微差别。"),
    w("empathy", "/ˈempəθi/", "共情；同理心", "Good listeners show empathy without judgment.", "好的倾听者不带评判地共情。"),
    w("hypothesis", "/haɪˈpɒθəsɪs/", "假说；假设", "The data supports our initial hypothesis.", "数据支持我们最初的假设。"),
    w("cognitive", "/ˈkɒɡnətɪv/", "认知的", "Reading improves cognitive ability.", "阅读提升认知能力。"),
    w("consensus", "/kənˈsensəs/", "共识", "The team reached a consensus quickly.", "团队很快达成了共识。"),
    w("diminish", "/dɪˈmɪnɪʃ/", "减少；减弱", "His influence began to diminish.", "他的影响力开始减弱。"),
    w("explicit", "/ɪkˈsplɪsɪt/", "明确的", "The instructions were explicit and easy to follow.", "说明明确且易于遵循。"),
    w("incentive", "/ɪnˈsentɪv/", "激励", "Bonus is a strong incentive for staff.", "奖金对员工是强大的激励。"),
    w("magnitude", "/ˈmæɡnɪtjuːd/", "巨大；量级", "We did not realize the magnitude of the problem.", "我们没有意识到问题的严重性。"),
];

const IELTS_WORDS: &[SeedWord] = &[
    w("alleviate", "/əˈliːvieɪt/", "缓解；减轻", "The medicine helps alleviate the pain.", "这种药有助于缓解疼痛。"),
    w("bureaucracy", "/bjʊəˈrɒkrəsi/", "官僚机构", "Too much bureaucracy slows down decisions.", "过多的官僚程序拖慢了决策。"),
    w("detrimental", "/ˌdetrɪˈmentl/", "有害的", "Smoking is detrimental to health.", "吸烟有害健康。"),
    w("exacerbate", "/ɪɡˈzæsəbeɪt/", "使恶化", "Stress can exacerbate the symptoms.", "压力会加剧症状。"),
    w("facilitate", "/fəˈsɪlɪteɪt/", "促进", "Technology facilitates global communication.", "技术促进全球交流。"),
    w("globalization", "/ˌɡləʊbəlaɪˈzeɪʃn/", "全球化", "Globalization has changed local economies.", "全球化改变了地方经济。"),
    w("homogeneous", "/ˌhɒməˈdʒiːniəs/", "同质的", "The population is fairly homogeneous.", "人口构成相当同质。"),
    w("infrastructure", "/ˈɪnfrəstrʌktʃə/", "基础设施", "The country invested heavily in infrastructure.", "该国大力投资基础设施。"),
    hw("juxtapose", "/ˌdʒʌkstəˈpəʊz/", "并列", "The essay juxtaposes two different viewpoints.", "文章并列了两种不同观点。"),
    w("mitigate", "/ˈmɪtɪɡeɪt/", "缓解；减轻", "We need measures to mitigate climate change.", "我们需要缓解气候变化的措施。"),
    w("phenomenon", "/fəˈnɒmɪnən/", "现象", "Urban migration is a global phenomenon.", "城市迁移是一种全球现象。"),
    w("prevalent", "/ˈprevələnt/", "普遍的", "This belief is prevalent in many cultures.", "这种信念在许多文化中都很普遍。"),
    hw("scrutinize", "/ˈskruːtənaɪz/", "仔细审查", "The report was scrutinized by experts.", "这份报告经过了专家的仔细审查。"),
    w("sustainability", "/səˌsteɪnəˈbɪləti/", "可持续性", "Sustainability is key to urban planning.", "可持续性是城市规划的关键。"),
    w("undermine", "/ˌʌndəˈmaɪn/", "削弱", "Late nights can undermine your health.", "熬夜会损害健康。"),
    w("advocate", "/ˈædvəkeɪt/", "提倡；拥护", "Many experts advocate a plant-based diet.", "许多专家提倡以植物为主的饮食。"),
    w("controversial", "/ˌkɒntrəˈvɜːʃl/", "有争议的", "The new policy is highly controversial.", "新政策争议很大。"),
    w("demographic", "/ˌdeməˈɡræfɪk/", "人口统计的", "The demographic shift affects the labor market.", "人口结构变化影响劳动力市场。"),
    w("fluctuate", "/ˈflʌktʃueɪt/", "波动", "Prices fluctuate with supply and demand.", "价格随供需波动。"),
    w("plausible", "/ˈplɔːzəbl/", "貌似合理的", "Her explanation sounds plausible.", "她的解释听起来有道理。"),
    w("serendipity", "/ˌserənˈdɪpəti/", "意外发现美好事物的能力", "Finding that old letter was pure serendipity.", "找到那封旧信完全是意外之喜。"),
    w("articulate", "/ɑːˈtɪkjuleɪt/", "清晰表达（的）", "He is articulate when explaining complex ideas.", "解释复杂想法时他表达得很清楚。"),
    w("coherent", "/kəʊˈhɪərənt/", "连贯的；有条理的", "The essay needs a more coherent structure.", "这篇文章需要更连贯的结构。"),
    w("viable", "/ˈvaɪəbl/", "可行的", "This plan is not economically viable.", "这个计划在经济上不可行。"),
    w("cognitive", "/ˈkɒɡnətɪv/", "认知的", "Reading improves cognitive ability.", "阅读提升认知能力。"),
    w("inevitable", "/ɪnˈevɪtəbl/", "不可避免的", "Change is inevitable in any growing city.", "任何成长中的城市都不可避免变化。"),
    w("paradigm", "/ˈpærədaɪm/", "范式", "The discovery created a new scientific paradigm.", "这一发现开创了新的科学范式。"),
    w("incentive", "/ɪnˈsentɪv/", "激励", "Bonus is a strong incentive for staff.", "奖金对员工是强大的激励。"),
];

const PROFESSIONAL_WORDS: &[SeedWord] = &[
    w("algorithm", "/ˈælɡərɪðəm/", "算法", "The algorithm sorts data in milliseconds.", "该算法在毫秒内完成数据排序。"),
    w("bandwidth", "/ˈbændwɪdθ/", "带宽", "The video needs high bandwidth.", "该视频需要高带宽。"),
    w("capacitor", "/kəˈpæsɪtə/", "电容器", "The capacitor stores electrical energy.", "电容器储存电能。"),
    w("depreciation", "/dɪˌpriːʃiˈeɪʃn/", "折旧", "Asset depreciation affects the balance sheet.", "资产折旧影响资产负债表。"),
    w("entropy", "/ˈentrəpi/", "熵", "Entropy measures disorder in a system.", "熵衡量系统的无序程度。"),
    w("fiscal", "/ˈfɪskl/", "财政的", "The government adopted a tight fiscal policy.", "政府采取了紧缩的财政政策。"),
    w("genome", "/ˈdʒiːnəʊm/", "基因组", "Scientists mapped the human genome.", "科学家绘制了人类基因组图谱。"),
    vw("hemodynamics", "/ˌhiːmədaɪˈnæmɪks/", "血流动力学", "Hemodynamics studies blood flow in vessels.", "血流动力学研究血管中的血液流动。"),
    w("jurisdiction", "/ˌdʒʊərɪsˈdɪkʃn/", "管辖权", "The court has jurisdiction over this case.", "法院对此案有管辖权。"),
    w("kinetic", "/kɪˈnetɪk/", "动能的", "Kinetic energy increases with speed.", "动能随速度增加而增加。"),
    w("liquidity", "/lɪˈkwɪdəti/", "流动性", "The bank maintains high liquidity.", "银行保持高流动性。"),
    w("molecule", "/ˈmɒlɪkjuːl/", "分子", "A water molecule contains two hydrogen atoms.", "一个水分子包含两个氢原子。"),
    w("neurology", "/njʊˈrɒlədʒi/", "神经学", "Neurology studies the nervous system.", "神经学研究神经系统。"),
    w("protocol", "/ˈprəʊtəkɒl/", "协议", "The devices communicate using a secure protocol.", "设备使用安全协议通信。"),
    w("quantum", "/ˈkwɒntəm/", "量子的", "Quantum computing is advancing rapidly.", "量子计算正在快速发展。"),
    w("revenue", "/ˈrevənjuː/", "收入", "Online sales drive most of the revenue.", "在线销售贡献了大部分收入。"),
    hw("semiconductor", "/ˌsemikənˈdʌktə/", "半导体", "Silicon is the main semiconductor material.", "硅是主要的半导体材料。"),
    vw("thermodynamics", "/ˌθɜːmədaɪˈnæmɪks/", "热力学", "Thermodynamics explains how engines convert heat.", "热力学解释发动机如何转换热量。"),
    w("venture", "/ˈventʃə/", "风险项目；冒险", "They launched a new tech venture.", "他们启动了一个新的科技项目。"),
    w("volatility", "/ˌvɒləˈtɪləti/", "波动性", "Market volatility scares small investors.", "市场波动吓坏了小投资者。"),
    w("derivative", "/dɪˈrɪvətɪv/", "导数；衍生品", "The derivative measures how a function changes.", "导数衡量函数的变化率。"),
    w("encryption", "/ɪnˈkrɪpʃn/", "加密", "Encryption protects sensitive data.", "加密保护敏感数据。"),
    w("forensic", "/fəˈrensɪk/", "法医的", "Forensic science helps solve crimes.", "法医科学帮助破案。"),
    w("leverage", "/ˈliːvərɪdʒ/", "杠杆；利用", "The firm uses leverage to expand.", "该公司利用杠杆扩张。"),
    w("metabolism", "/məˈtæbəlɪzəm/", "新陈代谢", "Exercise speeds up your metabolism.", "运动加快新陈代谢。"),
    w("portfolio", "/pɔːtˈfəʊliəʊ/", "投资组合", "A diversified portfolio reduces risk.", "多元化的投资组合降低风险。"),
];

pub const BOOKS: &[SeedBook] = &[
    SeedBook {
        code: "primary",
        title: "小学英语",
        description: "常见生活词汇，适合零基础到小学阶段。",
        level: "a1",
        difficulty: 0.7,
        sort_order: 1,
        words: PRIMARY_WORDS,
    },
    SeedBook {
        code: "junior",
        title: "初中英语",
        description: "中考核心词汇与常用表达。",
        level: "a2",
        difficulty: 0.9,
        sort_order: 2,
        words: JUNIOR_WORDS,
    },
    SeedBook {
        code: "senior",
        title: "高中英语",
        description: "高考高频词汇与进阶用法。",
        level: "b1",
        difficulty: 1.1,
        sort_order: 3,
        words: SENIOR_WORDS,
    },
    SeedBook {
        code: "college",
        title: "大学英语",
        description: "大学英语四六级核心词汇。",
        level: "b2",
        difficulty: 1.3,
        sort_order: 4,
        words: COLLEGE_WORDS,
    },
    SeedBook {
        code: "ielts",
        title: "雅思",
        description: "雅思考试高频学术词汇。",
        level: "c1",
        difficulty: 1.5,
        sort_order: 5,
        words: IELTS_WORDS,
    },
    SeedBook {
        code: "professional",
        title: "专业英语",
        description: "计算机、金融、医学等专业术语。",
        level: "c1",
        difficulty: 1.7,
        sort_order: 6,
        words: PROFESSIONAL_WORDS,
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seed_books_cover_six_levels() {
        assert_eq!(BOOKS.len(), 6);
        let codes: Vec<&str> = BOOKS.iter().map(|book| book.code).collect();
        assert!(codes.contains(&"primary"));
        assert!(codes.contains(&"ielts"));
        assert!(codes.contains(&"professional"));
    }

    #[test]
    fn every_book_has_words_and_phonetics() {
        for book in BOOKS {
            assert!(!book.words.is_empty(), "{} 词书不应为空", book.code);
            assert!(book.difficulty >= 0.5 && book.difficulty <= 2.0);
            for seed in book.words {
                assert!(!seed.word.is_empty());
                assert!(!seed.phonetic.is_empty());
                assert!(!seed.meaning.is_empty());
                assert!(seed.difficulty >= 0.5 && seed.difficulty <= 2.0);
            }
        }
    }

    #[test]
    fn difficulty_grows_with_stage() {
        let difficulties: Vec<f64> = BOOKS.iter().map(|book| book.difficulty).collect();
        let mut sorted = difficulties.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        assert_eq!(difficulties, sorted);
        assert!(difficulties[0] < difficulties[5]);
    }
}
